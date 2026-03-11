use crate::{geometry::Orientation, scroll_view_state::ScrollViewState, utils::Utils};
use ratatui::{Frame, layout::Rect, text::Line as RatatuiLine};
use std::ops::Range;

pub trait ScrollView {
    fn ratatui_lines(&self, rows: Range<usize>, cols: Range<usize>) -> Vec<RatatuiLine<'_>>;

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState;

    fn render_content(&mut self, frame: &mut Frame, content_area: Rect) {
        let scroll_offset = self.scroll_view_state_mut().scroll_offset();
        let rows = scroll_offset.y.range_from_len(content_area.height.into());
        let cols = scroll_offset.x.range_from_len(content_area.width.into());
        let ratatui_line_and_row_area_pairs = self.ratatui_lines(rows, cols).into_iter().zip(content_area.rows());

        for (ratatui_line, row_area) in ratatui_line_and_row_area_pairs {
            ratatui_line.render_to(frame, row_area);
        }
    }

    fn render_scroll_bars(&mut self, frame: &mut Frame, content_area: Rect) {
        let max_scroll_offset = self.scroll_view_state_mut().max_scroll_offset();

        if max_scroll_offset.x.is_positive() {
            self.scroll_view_state_mut()
                .scroll_bar(Orientation::Horizontal)
                .render(content_area, frame.buffer_mut());
        }

        if max_scroll_offset.y.is_positive() {
            self.scroll_view_state_mut()
                .scroll_bar(Orientation::Vertical)
                .render(content_area, frame.buffer_mut());
        }
    }

    fn render(&mut self, frame: &mut Frame, content_area: Rect) {
        let scroll_view_state = self.scroll_view_state_mut();

        scroll_view_state.set_latest_content_render_size(content_area.as_size().into());
        self.render_content(frame, content_area);
        self.render_scroll_bars(frame, content_area);
    }
}
