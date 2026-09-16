//! Fork-specific right-hand context retained separately from composer hints.

use super::ChatComposer;
use ratatui::text::Line;

impl ChatComposer {
    pub(crate) fn set_status_line_right(&mut self, status_line: Option<Line<'static>>) -> bool {
        if self.footer.status_line_right_value == status_line {
            return false;
        }
        self.footer.status_line_right_value = status_line;
        true
    }
}
