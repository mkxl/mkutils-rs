use crate::{
    content::Content,
    geometry::{Orientation, PointUsize},
    scroll_bar::ScrollBar,
    scrollable::{ScrollWhen, Scrollable},
    utils::Utils,
};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    widgets::{Block, Widget},
};
use std::borrow::Borrow;

pub struct ScrollView<T> {
    block: Block<'static>,
    content: T,
    scroll_offset: PointUsize,
    scroll_when: ScrollWhen,
    latest_content_size: PointUsize,
    latest_render_content_size: PointUsize,
}

impl<T: Content> ScrollView<T> {
    const INITIAL_SCROLL_OFFSET: PointUsize = PointUsize::ZERO;
    const INITIAL_LATEST_CONTENT_SIZE: PointUsize = PointUsize::ZERO;
    const INITIAL_LATEST_RENDER_CONTENT_SIZE: PointUsize = PointUsize::ZERO;
    const MARGIN_RENDER_CONTENT_AREA: Margin = Margin::new(1, 1);

    pub const fn new(content: T, scroll_when: ScrollWhen) -> Self {
        let block = Block::bordered();
        let scroll_offset = Self::INITIAL_SCROLL_OFFSET;
        let latest_content_size = Self::INITIAL_LATEST_CONTENT_SIZE;
        let latest_render_content_size = Self::INITIAL_LATEST_RENDER_CONTENT_SIZE;

        Self {
            block,
            content,
            scroll_offset,
            scroll_when,
            latest_content_size,
            latest_render_content_size,
        }
    }

    fn render_content(&self, render_content_area: Rect, buffer: &mut Buffer) {
        let rows = self.scroll_offset.y.range_from_len(render_content_area.height.into());
        let cols = self.scroll_offset.x.range_from_len(render_content_area.width.into());
        let line_and_row_area_pairs = self
            .content
            .lines(rows, cols)
            .into_iter()
            .zip(render_content_area.rows());

        for (line, row_area) in line_and_row_area_pairs {
            line.borrow().render(row_area, buffer);
        }
    }

    fn scroll_bar(&self, orientation: Orientation) -> ScrollBar {
        ScrollBar::new(
            self.scroll_offset,
            self.max_scroll_offset(),
            self.latest_content_size,
            orientation,
        )
    }

    fn render_scroll_bars(&self, render_content_area: Rect, buffer: &mut Buffer) {
        let max_scroll_offset = self.max_scroll_offset();

        if max_scroll_offset.x.is_positive() {
            self.scroll_bar(Orientation::Horizontal)
                .render(render_content_area, buffer);
        }

        if max_scroll_offset.y.is_positive() {
            self.scroll_bar(Orientation::Vertical)
                .render(render_content_area, buffer);
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

impl<T: Content> Scrollable for ScrollView<T> {
    fn latest_content_size(&self) -> PointUsize {
        self.latest_content_size
    }

    fn scroll_when(&self) -> ScrollWhen {
        self.scroll_when
    }

    fn scroll_offset_mut(&mut self) -> &mut PointUsize {
        &mut self.scroll_offset
    }

    fn latest_content_render_size(&self) -> PointUsize {
        self.latest_render_content_size
    }
}

impl<T: Content> Widget for &mut ScrollView<T> {
    fn render(self, render_area: Rect, buffer: &mut Buffer) {
        let render_content_area = render_area.inner(ScrollView::<T>::MARGIN_RENDER_CONTENT_AREA);

        self.latest_render_content_size = render_content_area.as_size().into();

        self.block.ref_immut().render(render_area, buffer);
        self.render_content(render_content_area, buffer);
        self.render_scroll_bars(render_content_area, buffer);
    }
}
