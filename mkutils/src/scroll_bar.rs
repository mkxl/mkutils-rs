use crate::{
    geometry::{Orientation, Point, PointUsize},
    transpose::Transpose,
    utils::Utils,
};
use derive_more::Constructor;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    symbols::line::{THICK_HORIZONTAL, THICK_VERTICAL},
};

#[derive(Constructor)]
pub struct ScrollBar {
    scroll_offset: PointUsize,
    max_scroll_offset: PointUsize,
    content_size: PointUsize,
    orientation: Orientation,
}

impl ScrollBar {
    const VERTICAL_THUMB_AREA_WIDTH: u16 = 1;
    const VERTICAL_THUMB_AREA_MIN_HEIGHT: u16 = 1;
    const SYMBOLS: Point<&'static str> = Point::new(THICK_HORIZONTAL, THICK_VERTICAL);

    fn vertical_thumb_area(&self, render_content_area: Rect) -> Rect {
        let thumb_area_height = render_content_area
            .height
            .interpolate(0, self.content_size.y, 0, render_content_area.height)
            .max(Self::VERTICAL_THUMB_AREA_MIN_HEIGHT);
        let max_thumb_area_y = render_content_area
            .bottom()
            .saturating_sub(thumb_area_height)
            .decremented();
        let thumb_area_y =
            self.scroll_offset
                .y
                .interpolate(0, self.max_scroll_offset.y, render_content_area.y, max_thumb_area_y);
        let thumb_area_x = render_content_area
            .right()
            .saturating_sub(Self::VERTICAL_THUMB_AREA_WIDTH);

        Rect::new(
            thumb_area_x,
            thumb_area_y.cast_or_max(),
            Self::VERTICAL_THUMB_AREA_WIDTH,
            thumb_area_height,
        )
    }

    fn horizontal_thumb_area(&self, render_content_area: Rect) -> Rect {
        self.transpose()
            .vertical_thumb_area(render_content_area.transpose())
            .transpose()
    }

    fn thumb_area(&self, render_content_area: Rect) -> Rect {
        match self.orientation {
            Orientation::Horizontal => self.horizontal_thumb_area(render_content_area),
            Orientation::Vertical => self.vertical_thumb_area(render_content_area),
        }
    }

    // NOTE: we intentionally don't implement [Widget] for [ScrollBar] using this
    // [render()] method as the supplied [Rect] argument should be for the area
    // of the content this is a scroll bar for and not for the area of the scroll
    // bar itself
    pub fn render(&self, render_content_area: Rect, buffer: &mut Buffer) {
        let symbol = Self::SYMBOLS.get(self.orientation);

        for position in self.thumb_area(render_content_area).positions() {
            if let Some(cell) = buffer.cell_mut(position) {
                cell.set_symbol(symbol);
            }
        }
    }
}

impl Transpose for ScrollBar {
    fn to_transpose(&self) -> Self {
        Self::new(
            self.scroll_offset.transpose(),
            self.max_scroll_offset.transpose(),
            self.content_size.transpose(),
            self.orientation.transpose(),
        )
    }
}
