use crate::geometry::PointUsize;
use mkutils_macros::Toggle;

#[derive(Toggle)]
pub enum ScrollWhen {
    Always,
    ForLargeContent,
}

#[derive(Clone, Copy)]
pub enum ScrollCount {
    Fixed(usize),
    PageSize,
}

pub trait Scrollable {
    fn scroll_offset_mut(&mut self) -> &mut PointUsize;

    fn latest_content_render_size(&self) -> PointUsize;

    fn content_size(&self) -> PointUsize;
}
