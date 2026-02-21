use crate::{
    geometry::{Orientation, PointUsize},
    scroll_bar::ScrollBar,
    utils::Utils,
};
use derive_more::From;
use mkutils_macros::Toggle;

#[derive(Clone, Copy, Toggle)]
pub enum ScrollWhen {
    Always,
    ForLargeContent,
}

#[derive(Clone, Copy, From)]
pub enum ScrollCountType {
    Fixed(usize),
    PageSize,
}

pub struct ScrollViewState {
    scroll_offset: PointUsize,
    scroll_when: ScrollWhen,
    latest_content_size: PointUsize,
    latest_content_render_size: PointUsize,
}

impl ScrollViewState {
    const INITIAL_SCROLL_OFFSET: PointUsize = PointUsize::ZERO;
    const INITIAL_LATEST_CONTENT_SIZE: PointUsize = PointUsize::ZERO;
    const INITIAL_LATEST_RENDER_CONTENT_SIZE: PointUsize = PointUsize::ZERO;

    #[must_use]
    pub const fn new(scroll_when: ScrollWhen) -> Self {
        let scroll_offset = Self::INITIAL_SCROLL_OFFSET;
        let latest_content_size = Self::INITIAL_LATEST_CONTENT_SIZE;
        let latest_content_render_size = Self::INITIAL_LATEST_RENDER_CONTENT_SIZE;

        Self {
            scroll_offset,
            scroll_when,
            latest_content_size,
            latest_content_render_size,
        }
    }

    fn scroll_count(&self, scroll_count_type: ScrollCountType, orientation: Orientation) -> usize {
        match scroll_count_type {
            ScrollCountType::Fixed(scroll_count) => scroll_count,
            ScrollCountType::PageSize => self.latest_content_render_size.get(orientation).copied(),
        }
    }

    #[must_use]
    pub fn max_scroll_offset(&self) -> PointUsize {
        match self.scroll_when {
            ScrollWhen::Always => self.latest_content_size.saturating_sub_scalar(&1),
            ScrollWhen::ForLargeContent => self
                .latest_content_size
                .saturating_sub(&self.latest_content_render_size),
        }
    }

    fn scroll(&mut self, scroll_count_type: ScrollCountType, orientation: Orientation, add: bool) {
        let scroll_count = self.scroll_count(scroll_count_type, orientation);
        let max_scroll_offset = self.max_scroll_offset().get(orientation).copied();

        self.scroll_offset
            .get_mut(orientation)
            .saturating_add_or_sub_in_place_with_max(scroll_count, max_scroll_offset, add);
    }

    #[must_use]
    pub const fn scroll_offset(&self) -> PointUsize {
        self.scroll_offset
    }

    #[must_use]
    pub const fn latest_content_size(&self) -> PointUsize {
        self.latest_content_size
    }

    pub const fn set_latest_content_size(&mut self, latest_content_size: PointUsize) {
        self.latest_content_size = latest_content_size;
    }

    pub const fn set_latest_content_render_size(&mut self, latest_content_render_size: PointUsize) {
        self.latest_content_render_size = latest_content_render_size;
    }

    pub fn scroll_down(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll(scroll_count_type.into(), Orientation::Vertical, true);
    }

    pub fn scroll_up(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll(scroll_count_type.into(), Orientation::Vertical, false);
    }

    pub fn scroll_left(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll(scroll_count_type.into(), Orientation::Horizontal, false);
    }

    pub fn scroll_right(&mut self, scroll_count_type: impl Into<ScrollCountType>) {
        self.scroll(scroll_count_type.into(), Orientation::Horizontal, true);
    }

    #[must_use]
    pub fn scroll_bar(&self, orientation: Orientation) -> ScrollBar {
        ScrollBar::new(
            self.scroll_offset(),
            self.max_scroll_offset(),
            self.latest_content_size(),
            orientation,
        )
    }
}
