use crate::{
    content::Content,
    geometry::{Orientation, PointUsize},
    scroll_view_state::ScrollViewState,
    utils::Utils,
};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    widgets::{Block, StatefulWidget, Widget},
};
use std::borrow::Borrow;

pub struct ScrollView<T> {
    block: Block<'static>,
    content: T,
    latest_content_size: PointUsize,
}

impl<T: Content> ScrollView<T> {
    const MARGIN_RENDER_CONTENT_AREA: Margin = Margin::new(1, 1);

    pub fn new(content: T) -> Self {
        let block = Block::bordered();
        let latest_content_size = content.size();

        Self {
            block,
            content,
            latest_content_size,
        }
    }

    fn render_content(&self, content_render_area: Rect, buffer: &mut Buffer, scroll_view_state: &ScrollViewState) {
        let scroll_offset = scroll_view_state.scroll_offset();
        let rows = scroll_offset.y.range_from_len(content_render_area.height.into());
        let cols = scroll_offset.x.range_from_len(content_render_area.width.into());
        let line_and_row_area_pairs = self
            .content
            .lines(rows, cols)
            .into_iter()
            .zip(content_render_area.rows());

        for (line, row_area) in line_and_row_area_pairs {
            line.borrow().render(row_area, buffer);
        }
    }

    fn render_scroll_bars(content_render_area: Rect, buffer: &mut Buffer, scroll_view_state: &ScrollViewState) {
        let max_scroll_offset = scroll_view_state.max_scroll_offset();

        if max_scroll_offset.x.is_positive() {
            scroll_view_state
                .scroll_bar(Orientation::Horizontal)
                .render(content_render_area, buffer);
        }

        if max_scroll_offset.y.is_positive() {
            scroll_view_state
                .scroll_bar(Orientation::Vertical)
                .render(content_render_area, buffer);
        }
    }

    pub fn set_block(&mut self, block: Block<'static>) {
        self.block = block;
    }

    pub fn set_content(&mut self, content: T) {
        self.latest_content_size = content.size();
        self.content = content;
    }
}

impl<T: Content> StatefulWidget for &ScrollView<T> {
    type State = ScrollViewState;

    fn render(self, render_area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let content_render_area = render_area.inner(ScrollView::<T>::MARGIN_RENDER_CONTENT_AREA);

        state.set_latest_content_size(self.latest_content_size);
        state.set_latest_content_render_size(content_render_area.as_size().into());
        self.block.ref_immut().render(render_area, buffer);
        self.render_content(content_render_area, buffer, state);
        ScrollView::<T>::render_scroll_bars(content_render_area, buffer, state);
    }
}
