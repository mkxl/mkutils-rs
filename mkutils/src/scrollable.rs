use crate::{
    geometry::{Orientation, PointUsize},
    utils::Utils,
};
use mkutils_macros::Toggle;

#[derive(Toggle)]
pub enum ScrollWhen {
    Always,
    ForLargeContent,
}

#[derive(Clone, Copy)]
pub enum ScrollCountType {
    Fixed(usize),
    PageSize,
}

pub trait Scrollable {
    fn latest_content_size(&self) -> PointUsize;

    fn scroll_when(&self) -> ScrollWhen;

    fn scroll_offset_mut(&mut self) -> &mut PointUsize;

    fn latest_content_render_size(&self) -> PointUsize;

    fn scroll_count(&self, scroll_count_type: ScrollCountType, orientation: Orientation) -> usize {
        match scroll_count_type {
            ScrollCountType::Fixed(scroll_count) => scroll_count,
            ScrollCountType::PageSize => self.latest_content_render_size().get(orientation).copied(),
        }
    }

    fn max_scroll_offset(&self) -> PointUsize {
        match self.scroll_when() {
            ScrollWhen::Always => self.latest_content_size().saturating_sub_scalar(&1),
            ScrollWhen::ForLargeContent => self
                .latest_content_size()
                .saturating_sub(&self.latest_content_render_size()),
        }
    }

    fn scroll(&mut self, scroll_count_type: ScrollCountType, orientation: Orientation, add: bool)
    where
        Self: Scrollable,
    {
        let scroll_count = self.scroll_count(scroll_count_type, orientation);
        let max_scroll_offset = self.max_scroll_offset().get(orientation).copied();

        self.scroll_offset_mut()
            .get_mut(orientation)
            .saturating_add_or_sub_in_place_with_max(scroll_count, max_scroll_offset, add);
    }

    fn scroll_down(&mut self, scroll_count_type: ScrollCountType) {
        self.scroll(scroll_count_type, Orientation::Vertical, true);
    }

    fn scroll_up(&mut self, scroll_count_type: ScrollCountType) {
        self.scroll(scroll_count_type, Orientation::Vertical, false);
    }

    fn scroll_left(&mut self, scroll_count_type: ScrollCountType) {
        self.scroll(scroll_count_type, Orientation::Horizontal, false);
    }

    fn scroll_right(&mut self, scroll_count_type: ScrollCountType) {
        self.scroll(scroll_count_type, Orientation::Horizontal, true);
    }
}
