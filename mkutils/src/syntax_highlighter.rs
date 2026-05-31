use crate::{alias_hash_map::AliasHashMap, utils::Utils};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};
use tree_sitter_highlight::{
    Error as TreeSitterError, Highlight as TreeSitterHighlight, HighlightConfiguration, HighlightEvent, Highlighter,
};

type CowStr = Cow<'static, str>;

pub struct ColorScheme<S> {
    default_style: S,
    style_from_capture_names: HashMap<CowStr, S>,
}

impl<S> ColorScheme<S> {
    const CAPTURE_NAME_SEPARATOR: &str = ".";

    pub fn new(default_style: S) -> Self {
        let style_from_capture_names = HashMap::new();

        Self {
            default_style,
            style_from_capture_names,
        }
    }

    #[must_use]
    pub fn insert(mut self, capture_name: impl Into<CowStr>, style: S) -> Self {
        self.style_from_capture_names.insert(capture_name.into(), style);

        self
    }

    #[must_use]
    pub fn insert_all(mut self, style_from_capture_names: impl IntoIterator<Item = (CowStr, S)>) -> Self {
        self.style_from_capture_names.extend(style_from_capture_names);

        self
    }

    const fn default_style(&self) -> &S {
        &self.default_style
    }

    fn get_style(&self, mut capture_name: &str) -> &S {
        loop {
            if let Some(style) = self.style_from_capture_names.get(capture_name) {
                return style;
            }

            let Some((parent_capture_name, _capture_name_component)) =
                capture_name.rsplit_once(Self::CAPTURE_NAME_SEPARATOR)
            else {
                return self.default_style();
            };

            capture_name = parent_capture_name;
        }
    }

    fn get_style_from_highlight_index<T: AsRef<str>>(
        &self,
        capture_names: &[T],
        highlight_capture_name_index: usize,
    ) -> &S {
        let Some(capture_name) = capture_names.get(highlight_capture_name_index) else {
            return self.default_style();
        };

        self.get_style(capture_name.as_ref())
    }
}

pub trait Highlight<S> {
    type Output;

    fn highlight(&mut self, begin_byte_index: usize, end_byte_index: usize, style: &S);
    fn finish(&mut self) -> Self::Output;
}

pub struct SyntaxHighlighter<S> {
    color_scheme: ColorScheme<S>,
    highlight_configuration_from_language_name: AliasHashMap<CowStr, HighlightConfiguration>,
    highlighter: Highlighter,
    highlight_capture_names: Vec<String>,
}

impl<S> SyntaxHighlighter<S> {
    fn reconfigure(&mut self) {
        self.highlight_capture_names = self
            .highlight_configuration_from_language_name
            .values()
            .flat_map(HighlightConfiguration::highlight_capture_names)
            .map(String::to_string)
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();

        for highlight_configuration in self.highlight_configuration_from_language_name.values_mut() {
            highlight_configuration.configure(&self.highlight_capture_names);
        }
    }

    pub fn add_languages(&mut self, highlight_configurations: impl IntoIterator<Item = HighlightConfiguration>) {
        for highlight_configuration in highlight_configurations {
            self.highlight_configuration_from_language_name.insert(
                highlight_configuration.language_name.clone().into_cow_owned(),
                highlight_configuration,
            );
        }

        self.reconfigure();
    }

    pub fn add_language(&mut self, highlight_configuration: HighlightConfiguration) {
        self.add_languages(highlight_configuration.once());
    }

    pub fn add_language_alias(&mut self, from_language_name: CowStr, to_language_name: CowStr) {
        self.highlight_configuration_from_language_name
            .insert_alias(from_language_name, to_language_name);
    }

    pub fn highlight<H: Highlight<S>>(
        &mut self,
        language_name: &str,
        source: &[u8],
        highlight: &mut H,
    ) -> Result<H::Output, TreeSitterError> {
        let Some(highlight_configuration) = self.highlight_configuration_from_language_name.get(language_name) else {
            return TreeSitterError::InvalidLanguage.err();
        };
        let highlight_events =
            self.highlighter
                .highlight(highlight_configuration, source, None, None, |injected_language_name| {
                    self.highlight_configuration_from_language_name
                        .get(injected_language_name)
                })?;
        let default_style = self.color_scheme.default_style();
        let mut style_stack = std::vec![default_style];

        for highlight_event in highlight_events {
            match highlight_event? {
                HighlightEvent::Source { start, end } => {
                    let style = style_stack.last().copied().unwrap_or(default_style);

                    highlight.highlight(start, end, style);
                }
                HighlightEvent::HighlightStart(TreeSitterHighlight(highlight_capture_name_index)) => {
                    self.color_scheme
                        .get_style_from_highlight_index(&self.highlight_capture_names, highlight_capture_name_index)
                        .push_to(style_stack.ref_mut());
                }
                HighlightEvent::HighlightEnd => style_stack.pop().mem_drop(),
            }
        }

        highlight.finish().ok()
    }
}
