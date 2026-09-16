//! Reserve a right-hand context value while reusing upstream status layout and painting.

use super::StatusLine;
use super::StatusLineFocus;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::layout::Size;
use ratatui::text::Line;
use ratatui::widgets::Widget;

impl StatusLine {
    pub(crate) fn height_with_right(&self, bounds: Size, right: &Line<'static>) -> u16 {
        if bounds.width == 0 || bounds.height == 0 {
            return 0;
        }
        let (left, _) = split_status_area(
            Rect::new(/*x*/ 0, /*y*/ 0, bounds.width, bounds.height),
            right,
        );
        self.height(left.as_size()).max(1)
    }

    pub(crate) fn render_with_right(
        &self,
        area: Rect,
        buf: &mut Buffer,
        focus: StatusLineFocus,
        right: &Line<'static>,
    ) {
        if area.is_empty() {
            return;
        }
        let (left, right_area) = split_status_area(area, right);
        self.render(left, buf, focus);
        right.render(right_area, buf);
    }
}

fn split_status_area(area: Rect, right: &Line<'static>) -> (Rect, Rect) {
    let right_width = right.width().min(usize::from(area.width)) as u16;
    let left = Rect {
        width: area
            .width
            .saturating_sub(right_width.saturating_add(u16::from(right_width > 0))),
        ..area
    };
    let right = Rect::new(
        area.right().saturating_sub(right_width),
        area.bottom().saturating_sub(1),
        right_width,
        u16::from(area.height > 0),
    );
    (left, right)
}
