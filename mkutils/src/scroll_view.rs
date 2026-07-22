use crate::{
    geometry::{Orientation, Point},
    scroll_view_state::{ScrollCountType, ScrollViewState},
    utils::Utils,
};
use crossterm::event::MouseEventKind;
use ratatui::{Frame, layout::Rect, style::Style};

pub trait ScrollView {
    fn scroll_view_state(&self) -> &ScrollViewState;

    fn scroll_view_state_mut(&mut self) -> &mut ScrollViewState;

    fn render_content(&self, frame: &mut Frame, content_area: Rect);

    fn render_misc(&self, _frame: &mut Frame, _scroll_view_area: Rect) {}

    fn scroll_bar_style(&self) -> Style {
        Style::default()
    }

    fn render_scroll_bars(&self, frame: &mut Frame, content_area: Rect) {
        let scroll_view_state = self.scroll_view_state();

        if !scroll_view_state.should_render_scroll_bars() {
            return;
        }

        let scroll_bar_style = self.scroll_bar_style();
        let max_scroll_offset = scroll_view_state.max_scroll_offset();

        if max_scroll_offset.x.is_positive() {
            scroll_view_state
                .scroll_bar(Orientation::Horizontal, scroll_bar_style)
                .render(content_area, frame.buffer_mut());
        }

        if max_scroll_offset.y.is_positive() {
            scroll_view_state
                .scroll_bar(Orientation::Vertical, scroll_bar_style)
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

    fn scroll_left(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll_view_state_mut().scroll_left(scroll_count_type);
    }

    fn scroll_right(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll_view_state_mut().scroll_right(scroll_count_type);
    }

    fn scroll_up(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll_view_state_mut().scroll_up(scroll_count_type);
    }

    fn scroll_down(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll_view_state_mut().scroll_down(scroll_count_type);
    }

    fn on_scroll(&mut self, mouse_event_kind: MouseEventKind, scroll_count_type: Point<impl Into<ScrollCountType>>) {
        match mouse_event_kind {
            MouseEventKind::ScrollLeft => self.scroll_left(scroll_count_type.x),
            MouseEventKind::ScrollRight => self.scroll_right(scroll_count_type.x),
            MouseEventKind::ScrollUp => self.scroll_up(scroll_count_type.y),
            MouseEventKind::ScrollDown => self.scroll_down(scroll_count_type.y),
            _ignored_mouse_event_kind => {}
        }
    }
}
