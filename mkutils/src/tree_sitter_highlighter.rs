use crate::{rope::rope::Rope, utils::Utils};
use anyhow::{Context, Error as AnyhowError, anyhow};
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use std::{collections::HashMap, ops::Range, sync::Mutex};
use tree_sitter::Language;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

const HIGHLIGHT_NAMES: [&str; 24] = [
    "none",
    "attribute",
    "character",
    "comment",
    "constant",
    "constructor",
    "function",
    "keyword",
    "markup.raw",
    "number",
    "operator",
    "property",
    "punctuation",
    "string.escape",
    "string",
    "text.emphasis",
    "text.literal",
    "text.reference",
    "text.strong",
    "text.title",
    "text.uri",
    "type",
    "variable",
    "warning",
];

pub struct TreeSitterHighlightTheme {
    default_style: Style,
    styles: HashMap<&'static str, Style>,
}

impl TreeSitterHighlightTheme {
    #[must_use]
    pub fn new(default_style: Style) -> Self {
        let styles = HashMap::new();

        Self { default_style, styles }
    }

    #[must_use]
    pub fn with_style(mut self, capture_name: &'static str, style: Style) -> Self {
        self.styles.insert(capture_name, style);

        self
    }

    fn style_for(&self, capture_name: &str) -> Style {
        let mut capture_name = capture_name;

        loop {
            if let Some(style) = self.styles.get(capture_name) {
                return *style;
            }

            let Some((prefix, _suffix)) = capture_name.rsplit_once('.') else {
                return self.default_style;
            };

            capture_name = prefix;
        }
    }

    fn style_for_highlight_index(&self, highlight_index: usize) -> Style {
        HIGHLIGHT_NAMES
            .get(highlight_index)
            .map_or(self.default_style, |capture_name| self.style_for(capture_name))
    }
}

struct TreeSitterHighlightConfig {
    name: &'static str,
    language: Language,
    highlights_query: &'static str,
    injections_query: &'static str,
    locals_query: &'static str,
}

impl TreeSitterHighlightConfig {
    #[must_use]
    const fn new(
        name: &'static str,
        language: Language,
        highlights_query: &'static str,
        injections_query: &'static str,
        locals_query: &'static str,
    ) -> Self {
        Self {
            name,
            language,
            highlights_query,
            injections_query,
            locals_query,
        }
    }

    fn into_highlight_configuration(self) -> Result<HighlightConfiguration, AnyhowError> {
        let mut config = HighlightConfiguration::new(
            self.language,
            self.name,
            self.highlights_query,
            self.injections_query,
            self.locals_query,
        )?;

        config.configure(&HIGHLIGHT_NAMES);

        config.ok()
    }
}

struct HighlightSource<'a> {
    source: &'a str,
    rope: Rope,
    line_index: usize,
    line_end_byte: usize,
}

impl<'a> HighlightSource<'a> {
    fn new(source: &'a str) -> Self {
        let rope = Rope::from(source);
        let line_index = 0;
        let line_end_byte = Self::line_end_byte(&rope, line_index, source.len());

        Self {
            source,
            rope,
            line_index,
            line_end_byte,
        }
    }

    fn line_end_byte(rope: &Rope, line_index: usize, source_len: usize) -> usize {
        rope.line_info(line_index)
            .map_or(source_len, |line_info| line_info.end.length.bytes)
    }

    fn advance_line(&mut self) {
        self.line_index = self.line_index.incremented();
        self.line_end_byte = Self::line_end_byte(&self.rope, self.line_index, self.source.len());
    }

    fn push_range(
        &mut self,
        lines: &mut Vec<Line<'static>>,
        spans: &mut Vec<Span<'static>>,
        range: Range<usize>,
        style: Style,
    ) {
        let mut start = range.start;

        while start < range.end {
            let end = range.end.min(self.line_end_byte);
            let segment = &self.source[start..end];

            if let Some(line_segment) = segment.strip_suffix('\n') {
                spans.push(Span::styled(line_segment.to_owned(), style));
                lines.push(std::mem::take(spans).into());
                self.advance_line();
            } else {
                spans.push(Span::styled(segment.to_owned(), style));
            }

            start = end;
        }
    }
}

pub struct RatatuiTreeSitterHighlighter {
    theme: TreeSitterHighlightTheme,
    configs: HashMap<&'static str, HighlightConfiguration>,
    aliases: HashMap<&'static str, &'static str>,
    highlighter: Mutex<Highlighter>,
}

impl RatatuiTreeSitterHighlighter {
    /// Creates a highlighter with the built-in markdown and Lean highlight configs.
    ///
    /// # Panics
    ///
    /// Panics if a built-in Tree-sitter highlight configuration is invalid.
    #[must_use]
    pub fn new(theme: TreeSitterHighlightTheme) -> Self {
        let configs = HashMap::new();
        let aliases = HashMap::new();
        let highlighter = Mutex::new(Highlighter::new());
        let mut this = Self {
            theme,
            configs,
            aliases,
            highlighter,
        };

        this.register_builtin_languages()
            .expect("built-in Tree-sitter highlight configs should be valid");

        this
    }

    fn register_builtin_languages(&mut self) -> Result<(), AnyhowError> {
        self.register_language(TreeSitterHighlightConfig::new(
            "markdown",
            tree_sitter_md::LANGUAGE.into(),
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            tree_sitter_md::INJECTION_QUERY_BLOCK,
            "",
        ))?;
        self.register_language(TreeSitterHighlightConfig::new(
            "markdown_inline",
            tree_sitter_md::INLINE_LANGUAGE.into(),
            tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
            tree_sitter_md::INJECTION_QUERY_INLINE,
            "",
        ))?;
        self.register_language(TreeSitterHighlightConfig::new(
            "lean",
            arborium_lean::language().into(),
            arborium_lean::HIGHLIGHTS_QUERY,
            arborium_lean::INJECTIONS_QUERY,
            arborium_lean::LOCALS_QUERY,
        ))?;
        self.alias_language("lean4", "lean");

        ().ok()
    }

    fn register_language(&mut self, config: TreeSitterHighlightConfig) -> Result<(), AnyhowError> {
        self.configs.insert(config.name, config.into_highlight_configuration()?);

        ().ok()
    }

    fn alias_language(&mut self, alias: &'static str, language_name: &'static str) {
        self.aliases.insert(alias, language_name);
    }

    fn resolve_language_name<'a>(&'a self, language_name: &'a str) -> &'a str {
        self.aliases.get(language_name).copied().unwrap_or(language_name)
    }

    fn config(&self, language_name: &str) -> Option<&HighlightConfiguration> {
        self.configs.get(self.resolve_language_name(language_name))
    }

    pub fn highlight(&self, language_name: &str, source: &str) -> Result<Vec<Line<'static>>, AnyhowError> {
        let config = self
            .config(language_name)
            .context("unknown Tree-sitter highlight language")?;
        let events = {
            let mut highlighter = self
                .highlighter
                .lock()
                .map_err(|_error| anyhow!("Tree-sitter highlighter mutex is poisoned"))?;
            highlighter
                .highlight(config, source.as_bytes(), None, |injected_language_name| {
                    self.config(injected_language_name)
                })?
                .collect::<Result<Vec<_>, _>>()?
        };
        let mut lines = Vec::new();
        let mut spans = Vec::new();
        let mut source = HighlightSource::new(source);
        let mut style_stack = vec![self.theme.default_style];

        for event in events {
            match event {
                HighlightEvent::Source { start, end } => {
                    let style = style_stack.last().copied().unwrap_or(self.theme.default_style);

                    source.push_range(&mut lines, &mut spans, start..end, style);
                }
                HighlightEvent::HighlightStart(highlight) => {
                    style_stack.push(self.theme.style_for_highlight_index(highlight.0));
                }
                HighlightEvent::HighlightEnd => {
                    style_stack.pop();
                }
            }
        }

        if !spans.is_empty() {
            lines.push(spans.into());
        }

        lines.ok()
    }
}
