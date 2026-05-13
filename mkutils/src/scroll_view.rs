use crate::{geometry::Orientation, scroll_view_state::ScrollViewState, utils::Utils};
use ratatui::{Frame, layout::Rect};

pub trait ScrollView {
    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState;

    fn render_content(&mut self, frame: &mut Frame, content_area: Rect);

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
