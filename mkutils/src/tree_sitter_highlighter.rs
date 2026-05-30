use crate::{
    rope::{rope::Rope, text_summary::LengthBytes},
    utils::Utils,
};
use anyhow::{Context, Error as AnyhowError, anyhow};
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use std::{collections::HashMap, ops::Range, sync::Mutex};
use tree_sitter::{
    Language, Node, Parser, Point, Query, QueryCursor, Range as TreeSitterRange, StreamingIterator, TextProvider,
};
use zed_sum_tree::Bias;

pub struct TreeSitterHighlightTheme {
    default_style: Style,
    styles: HashMap<&'static str, Style>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn highlights_injected_lean_fenced_code() {
        let keyword_style = Style::new().fg(Color::Red);
        let highlighter = RatatuiTreeSitterHighlighter::new(
            TreeSitterHighlightTheme::new(Style::new()).with_style("keyword", keyword_style),
        );
        let lines = highlighter.highlight("markdown", "```lean\n#check Nat\n```").unwrap();
        let lean_lines = highlighter.highlight("lean", "#check Nat\n").unwrap();

        assert!(
            lines
                .iter()
                .flat_map(|line| &line.spans)
                .any(|span| { span.content.contains("#check") && span.style == keyword_style }),
            "markdown: {lines:#?}\nlean: {lean_lines:#?}"
        );
    }
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

    fn style_for_highlight_index(&self, highlight_names: &[String], highlight_index: usize) -> Style {
        highlight_names
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

    fn capture_names(&self) -> Result<Vec<String>, AnyhowError> {
        Query::new(&self.language, self.highlights_query)
            .map(|query| query.capture_names().iter().map(ToString::to_string).collect())
            .map_err(AnyhowError::from)
    }

    fn into_highlight_configuration(
        self,
        highlight_names: &[String],
    ) -> Result<RegisteredTreeSitterHighlightConfig, AnyhowError> {
        let mut query_source = String::new();
        query_source.push_str(self.injections_query);
        let locals_query_offset = query_source.len();
        query_source.push_str(self.locals_query);
        let highlights_query_offset = query_source.len();
        query_source.push_str(self.highlights_query);

        let mut query = Query::new(&self.language, &query_source)?;
        let mut locals_pattern_index = 0;
        let mut highlights_pattern_index = 0;

        for pattern_index in 0..query.pattern_count() {
            let pattern_offset = query.start_byte_for_pattern(pattern_index);

            if pattern_offset < highlights_query_offset {
                highlights_pattern_index += 1;
            }

            if pattern_offset < locals_query_offset {
                locals_pattern_index += 1;
            }
        }

        let mut combined_injections_query = Query::new(&self.language, self.injections_query)?;
        let mut has_combined_queries = false;

        for pattern_index in 0..locals_pattern_index {
            let settings = query.property_settings(pattern_index);

            if settings
                .iter()
                .any(|setting| setting.key.as_ref() == "injection.combined")
            {
                has_combined_queries = true;
                query.disable_pattern(pattern_index);
            } else {
                combined_injections_query.disable_pattern(pattern_index);
            }
        }

        let combined_injections_query = has_combined_queries.then_some(combined_injections_query);
        let highlight_indices = query
            .capture_names()
            .iter()
            .map(|capture_name| {
                highlight_names
                    .iter()
                    .position(|highlight_name| highlight_name == capture_name)
            })
            .collect();
        let mut injection_content_capture_index = None;
        let mut injection_language_capture_index = None;

        for (capture_index, capture_name) in query.capture_names().iter().enumerate() {
            match *capture_name {
                "injection.content" => injection_content_capture_index = u32::try_from(capture_index).ok(),
                "injection.language" => injection_language_capture_index = u32::try_from(capture_index).ok(),
                _ => {}
            }
        }

        RegisteredTreeSitterHighlightConfig {
            language_name: self.name,
            language: self.language,
            query,
            combined_injections_query,
            locals_pattern_index,
            highlights_pattern_index,
            highlight_indices,
            injection_content_capture_index,
            injection_language_capture_index,
        }
        .ok()
    }
}

struct RegisteredTreeSitterHighlightConfig {
    language_name: &'static str,
    language: Language,
    query: Query,
    combined_injections_query: Option<Query>,
    locals_pattern_index: usize,
    highlights_pattern_index: usize,
    highlight_indices: Vec<Option<usize>>,
    injection_content_capture_index: Option<u32>,
    injection_language_capture_index: Option<u32>,
}

impl RegisteredTreeSitterHighlightConfig {
    fn highlight_index_for_capture(&self, capture_index: u32) -> Option<usize> {
        self.highlight_indices.get(capture_index as usize).copied().flatten()
    }
}

#[derive(Clone)]
struct RopeTextProvider<'r> {
    rope: &'r Rope,
}

impl<'r> RopeTextProvider<'r> {
    const fn new(rope: &'r Rope) -> Self {
        Self { rope }
    }
}

impl<'r> TextProvider<&'r str> for RopeTextProvider<'r> {
    type I = std::vec::IntoIter<&'r str>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        rope_slices_in_range(self.rope, node.byte_range()).into_iter()
    }
}

#[derive(Clone, Debug)]
struct HighlightRange {
    range: Range<usize>,
    highlight_index: usize,
    depth: usize,
}

#[derive(Clone)]
struct InjectionRequest {
    language_name: String,
    ranges: Vec<TreeSitterRange>,
    depth: usize,
}

struct TreeSitterHighlightState {
    parser: Parser,
    cursor: QueryCursor,
}

impl TreeSitterHighlightState {
    fn new() -> Self {
        Self {
            parser: Parser::new(),
            cursor: QueryCursor::new(),
        }
    }
}

struct HighlightSource<'r> {
    rope: &'r Rope,
    line_index: usize,
    line_end_byte: usize,
}

impl<'r> HighlightSource<'r> {
    fn new(rope: &'r Rope) -> Self {
        let line_index = 0;
        let line_end_byte = Self::line_end_byte(rope, line_index);

        Self {
            rope,
            line_index,
            line_end_byte,
        }
    }

    fn line_end_byte(rope: &Rope, line_index: usize) -> usize {
        rope.line_info(line_index)
            .map_or_else(|| rope.length().bytes, |line_info| line_info.end.length.bytes)
    }

    fn advance_line(&mut self) {
        self.line_index = self.line_index.incremented();
        self.line_end_byte = Self::line_end_byte(self.rope, self.line_index);
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

            for segment in rope_slices_in_range(self.rope, start..end) {
                if let Some(line_segment) = segment.strip_suffix('\n') {
                    spans.push(Span::styled(line_segment.to_owned(), style));
                    lines.push(std::mem::take(spans).into());
                    self.advance_line();
                } else {
                    spans.push(Span::styled(segment.to_owned(), style));
                }
            }

            start = end;
        }
    }
}

fn rope_slice_from_byte(rope: &Rope, byte_offset: usize) -> &str {
    let source_len = rope.length().bytes;

    if source_len <= byte_offset {
        return "";
    }

    let mut cursor = rope.chunk_sum_tree().cursor::<LengthBytes>(Rope::CONTEXT);
    cursor.seek(&LengthBytes::new(byte_offset), Bias::Right);

    let Some(chunk) = cursor.item() else {
        return "";
    };

    let chunk_start = cursor.start().get();
    let chunk_byte_offset = byte_offset.saturating_sub(chunk_start);

    &chunk.as_str()[chunk_byte_offset..]
}

fn rope_slices_in_range(rope: &Rope, range: Range<usize>) -> Vec<&str> {
    let source_len = rope.length().bytes;
    let range = range.start.min(source_len)..range.end.min(source_len);
    let mut slices = Vec::new();

    if range.is_empty() {
        return slices;
    }

    let mut cursor = rope.chunk_sum_tree().cursor::<LengthBytes>(Rope::CONTEXT);
    cursor.seek(&LengthBytes::new(range.start), Bias::Right);

    while let Some(chunk) = cursor.item() {
        let chunk_start = cursor.start().get();
        let chunk_end = chunk_start.saturating_add(chunk.len_bytes());

        if range.end <= chunk_start {
            break;
        }

        let start = range.start.saturating_sub(chunk_start).min(chunk.len_bytes());
        let end = range.end.min(chunk_end).saturating_sub(chunk_start);

        for segment in chunk.as_str()[start..end].split_inclusive('\n') {
            if !segment.is_empty() {
                slices.push(segment);
            }
        }

        if range.end <= chunk_end {
            break;
        }

        cursor.next();
    }

    slices
}

fn rope_text_in_range(rope: &Rope, range: Range<usize>) -> String {
    rope_slices_in_range(rope, range).concat()
}

fn root_range(rope: &Rope) -> TreeSitterRange {
    TreeSitterRange {
        start_byte: 0,
        start_point: Point::new(0, 0),
        end_byte: rope.length().bytes,
        end_point: Point::new(usize::MAX, usize::MAX),
    }
}

fn intersect_ranges(
    parent_ranges: &[TreeSitterRange],
    nodes: &[Node],
    includes_children: bool,
) -> Vec<TreeSitterRange> {
    let Some(first_node) = nodes.first() else {
        return Vec::new();
    };
    let mut cursor = first_node.walk();
    let mut result = Vec::new();
    let mut parent_range_iter = parent_ranges.iter();
    let Some(mut parent_range) = parent_range_iter.next() else {
        return result;
    };

    for node in nodes {
        let mut preceding_range = TreeSitterRange {
            start_byte: 0,
            start_point: Point::new(0, 0),
            end_byte: node.start_byte(),
            end_point: node.start_position(),
        };
        let following_range = TreeSitterRange {
            start_byte: node.end_byte(),
            start_point: node.end_position(),
            end_byte: usize::MAX,
            end_point: Point::new(usize::MAX, usize::MAX),
        };

        for excluded_range in node
            .children(&mut cursor)
            .filter(|_child| !includes_children)
            .map(|child| child.range())
            .chain(std::iter::once(following_range))
        {
            let mut range = TreeSitterRange {
                start_byte: preceding_range.end_byte,
                start_point: preceding_range.end_point,
                end_byte: excluded_range.start_byte,
                end_point: excluded_range.start_point,
            };
            preceding_range = excluded_range;

            if range.end_byte < parent_range.start_byte {
                continue;
            }

            while parent_range.start_byte <= range.end_byte {
                if parent_range.end_byte > range.start_byte {
                    if range.start_byte < parent_range.start_byte {
                        range.start_byte = parent_range.start_byte;
                        range.start_point = parent_range.start_point;
                    }

                    if parent_range.end_byte < range.end_byte {
                        if range.start_byte < parent_range.end_byte {
                            result.push(TreeSitterRange {
                                start_byte: range.start_byte,
                                start_point: range.start_point,
                                end_byte: parent_range.end_byte,
                                end_point: parent_range.end_point,
                            });
                        }
                        range.start_byte = parent_range.end_byte;
                        range.start_point = parent_range.end_point;
                    } else {
                        if range.start_byte < range.end_byte {
                            result.push(range);
                        }
                        break;
                    }
                }

                if let Some(next_range) = parent_range_iter.next() {
                    parent_range = next_range;
                } else {
                    return result;
                }
            }
        }
    }

    result
}

pub struct RatatuiTreeSitterHighlighter {
    theme: TreeSitterHighlightTheme,
    highlight_names: Vec<String>,
    configs: HashMap<&'static str, RegisteredTreeSitterHighlightConfig>,
    aliases: HashMap<&'static str, &'static str>,
    state: Mutex<TreeSitterHighlightState>,
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
        let state = Mutex::new(TreeSitterHighlightState::new());
        let mut this = Self {
            theme,
            highlight_names: Vec::new(),
            configs,
            aliases,
            state,
        };

        this.register_builtin_languages()
            .expect("built-in Tree-sitter highlight configs should be valid");

        this
    }

    fn register_builtin_languages(&mut self) -> Result<(), AnyhowError> {
        let configs = [
            TreeSitterHighlightConfig::new(
                "markdown",
                tree_sitter_md::LANGUAGE.into(),
                tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
                tree_sitter_md::INJECTION_QUERY_BLOCK,
                "",
            ),
            TreeSitterHighlightConfig::new(
                "markdown_inline",
                tree_sitter_md::INLINE_LANGUAGE.into(),
                tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
                tree_sitter_md::INJECTION_QUERY_INLINE,
                "",
            ),
            TreeSitterHighlightConfig::new(
                "lean",
                arborium_lean::language().into(),
                arborium_lean::HIGHLIGHTS_QUERY,
                arborium_lean::INJECTIONS_QUERY,
                arborium_lean::LOCALS_QUERY,
            ),
        ];

        self.highlight_names = Self::capture_names(&configs)?;

        for config in configs {
            self.configs
                .insert(config.name, config.into_highlight_configuration(&self.highlight_names)?);
        }

        self.alias_language("lean4", "lean");

        ().ok()
    }

    fn capture_names(configs: &[TreeSitterHighlightConfig]) -> Result<Vec<String>, AnyhowError> {
        let mut capture_names = Vec::new();

        for config in configs {
            for capture_name in config.capture_names()? {
                if !capture_names.iter().any(|name| name == &capture_name) {
                    capture_names.push(capture_name);
                }
            }
        }

        capture_names.ok()
    }

    fn alias_language(&mut self, alias: &'static str, language_name: &'static str) {
        self.aliases.insert(alias, language_name);
    }

    fn resolve_language_name<'a>(&'a self, language_name: &'a str) -> &'a str {
        self.aliases.get(language_name).copied().unwrap_or(language_name)
    }

    fn config(&self, language_name: &str) -> Option<&RegisteredTreeSitterHighlightConfig> {
        self.configs.get(self.resolve_language_name(language_name))
    }

    pub fn highlight(&self, language_name: &str, source: &str) -> Result<Vec<Line<'static>>, AnyhowError> {
        let rope = Rope::from(source);

        self.highlight_rope(language_name, &rope)
    }

    pub fn highlight_rope(&self, language_name: &str, rope: &Rope) -> Result<Vec<Line<'static>>, AnyhowError> {
        let config = self
            .config(language_name)
            .context("unknown Tree-sitter highlight language")?;
        let mut state = self
            .state
            .lock()
            .map_err(|_error| anyhow!("Tree-sitter highlighter mutex is poisoned"))?;
        let mut highlight_ranges = Vec::new();

        self.collect_highlight_ranges(
            &mut state,
            rope,
            config,
            None,
            vec![root_range(rope)],
            0,
            &mut highlight_ranges,
        )?;

        drop(state);

        self.highlight_ranges_to_lines(rope, &highlight_ranges)
    }

    #[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
    fn collect_highlight_ranges(
        &self,
        state: &mut TreeSitterHighlightState,
        rope: &Rope,
        config: &RegisteredTreeSitterHighlightConfig,
        parent_name: Option<&str>,
        ranges: Vec<TreeSitterRange>,
        depth: usize,
        highlight_ranges: &mut Vec<HighlightRange>,
    ) -> Result<(), AnyhowError> {
        state.parser.set_language(&config.language)?;
        state.parser.set_included_ranges(&ranges)?;
        let tree = state
            .parser
            .parse_with_options(
                &mut |byte_offset, _point| rope_slice_from_byte(rope, byte_offset),
                None,
                None,
            )
            .context("Tree-sitter parser did not produce a tree")?;
        let mut injections = Vec::new();

        if let Some(combined_injections_query) = config.combined_injections_query.as_ref() {
            let mut injections_by_pattern_index =
                vec![(None::<String>, Vec::new(), false); combined_injections_query.pattern_count()];
            let mut matches =
                state
                    .cursor
                    .matches(combined_injections_query, tree.root_node(), RopeTextProvider::new(rope));

            while let Some(query_match) = matches.next() {
                let entry = &mut injections_by_pattern_index[query_match.pattern_index];
                let (language_name, content_node, include_children) =
                    Self::injection_for_match(config, parent_name, combined_injections_query, query_match, rope);

                if language_name.is_some() {
                    entry.0 = language_name;
                }
                if let Some(content_node) = content_node {
                    entry.1.push(content_node);
                }
                entry.2 = include_children;
            }

            for (language_name, content_nodes, includes_children) in injections_by_pattern_index {
                if let Some(language_name) = language_name
                    && !content_nodes.is_empty()
                {
                    let ranges = intersect_ranges(&ranges, &content_nodes, includes_children);

                    if !ranges.is_empty() {
                        injections.push(InjectionRequest {
                            language_name,
                            ranges,
                            depth: depth.incremented(),
                        });
                    }
                }
            }
        }

        {
            let mut seen_injection_matches = Vec::new();
            let mut captures = state
                .cursor
                .captures(&config.query, tree.root_node(), RopeTextProvider::new(rope));

            captures.advance();
            while let Some((query_match, capture_index)) = captures.get() {
                let capture = query_match.captures[*capture_index];

                if query_match.pattern_index < config.locals_pattern_index {
                    if !seen_injection_matches.contains(&query_match.id()) {
                        seen_injection_matches.push(query_match.id());
                        let (language_name, content_node, include_children) =
                            Self::injection_for_match(config, parent_name, &config.query, query_match, rope);

                        if let (Some(language_name), Some(content_node)) = (language_name, content_node) {
                            let ranges = intersect_ranges(&ranges, &[content_node], include_children);

                            if !ranges.is_empty() {
                                injections.push(InjectionRequest {
                                    language_name,
                                    ranges,
                                    depth: depth.incremented(),
                                });
                            }
                        }
                    }
                } else if query_match.pattern_index >= config.highlights_pattern_index
                    && let Some(highlight_index) = config.highlight_index_for_capture(capture.index)
                {
                    let range = capture.node.byte_range();

                    if !range.is_empty() {
                        highlight_ranges.push(HighlightRange {
                            range,
                            highlight_index,
                            depth,
                        });
                    }
                }

                captures.advance();
            }
        }

        drop(tree);

        for injection in injections {
            if let Some(injection_config) = self.config(&injection.language_name) {
                self.collect_highlight_ranges(
                    state,
                    rope,
                    injection_config,
                    Some(config.language_name),
                    injection.ranges,
                    injection.depth,
                    highlight_ranges,
                )?;
            }
        }

        Ok(())
    }

    fn injection_for_match<'tree>(
        config: &RegisteredTreeSitterHighlightConfig,
        parent_name: Option<&str>,
        query: &Query,
        query_match: &tree_sitter::QueryMatch<'_, 'tree>,
        rope: &Rope,
    ) -> (Option<String>, Option<Node<'tree>>, bool) {
        let mut language_name = None;
        let mut content_node = None;

        for capture in query_match.captures {
            let index = Some(capture.index);

            if index == config.injection_language_capture_index {
                language_name = Some(rope_text_in_range(rope, capture.node.byte_range()));
            } else if index == config.injection_content_capture_index {
                content_node = Some(capture.node);
            }
        }

        let mut include_children = false;

        for property in query.property_settings(query_match.pattern_index) {
            match property.key.as_ref() {
                "injection.language" if language_name.is_none() => {
                    language_name = property.value.as_ref().map(ToString::to_string);
                }
                "injection.self" if language_name.is_none() => {
                    language_name = Some(config.language_name.to_owned());
                }
                "injection.parent" if language_name.is_none() => {
                    language_name = parent_name.map(ToString::to_string);
                }
                "injection.include-children" => include_children = true,
                _ => {}
            }
        }

        if let Some(content_node) = content_node
            && content_node.kind() == "code_fence_content"
        {
            include_children = true;
        }

        (language_name, content_node, include_children)
    }

    fn highlight_ranges_to_lines(
        &self,
        rope: &Rope,
        highlight_ranges: &[HighlightRange],
    ) -> Result<Vec<Line<'static>>, AnyhowError> {
        let mut boundaries = vec![0, rope.length().bytes];

        for highlight_range in highlight_ranges {
            boundaries.push(highlight_range.range.start);
            boundaries.push(highlight_range.range.end);
        }

        boundaries.sort_unstable();
        boundaries.dedup();

        let mut lines = Vec::new();
        let mut spans = Vec::new();
        let mut source = HighlightSource::new(rope);

        for window in boundaries.windows(2) {
            let [start, end] = window else {
                continue;
            };

            if start == end {
                continue;
            }

            let style = highlight_ranges
                .iter()
                .filter(|highlight_range| highlight_range.range.start <= *start && *end <= highlight_range.range.end)
                .max_by_key(|highlight_range| {
                    (
                        highlight_range.depth,
                        usize::MAX
                            .saturating_sub(highlight_range.range.end.saturating_sub(highlight_range.range.start)),
                    )
                })
                .map_or(self.theme.default_style, |highlight_range| {
                    self.theme
                        .style_for_highlight_index(&self.highlight_names, highlight_range.highlight_index)
                });

            source.push_range(&mut lines, &mut spans, *start..*end, style);
        }

        if !spans.is_empty() {
            lines.push(spans.into());
        }

        lines.ok()
    }
}
