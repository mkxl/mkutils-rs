use crate::{geometry::Orientation, scroll_view_state::ScrollViewState, utils::Utils};
use ratatui::{Frame, layout::Rect};

pub trait ScrollView {
    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState;

    fn render_content(&self, frame: &mut Frame, content_area: Rect);

    fn render_misc(&self, _frame: &mut Frame, _scroll_view_area: Rect) {}

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

    fn content_area(&self, scroll_view_area: Rect) -> Rect {
        scroll_view_area
    }

    fn render(&mut self, frame: &mut Frame, scroll_view_area: Rect) {
        let content_area = self.content_area(scroll_view_area);
        let scroll_view_state = self.scroll_view_state_mut();

        scroll_view_state.set_latest_scroll_view_area_size(scroll_view_area.as_size().into());
        self.render_content(frame, content_area);
        self.render_scroll_bars(frame, content_area);
        self.render_misc(frame, scroll_view_area);
    }
}
