//! Fork-specific context value shared by the inline and fullscreen status surfaces.

use super::ChatComposer;
use ratatui::style::Stylize;
use ratatui::text::Line;

impl ChatComposer {
    pub(super) fn append_status_line_right_value(
        &self,
        line: Option<Line<'static>>,
    ) -> Option<Line<'static>> {
        if !self.footer.status_line_enabled {
            return line;
        }
        let Some(status_line_right) = self.footer.status_line_right_value.clone() else {
            return line;
        };

        if let Some(mut line) = line {
            if line.width() > 0 {
                line.spans.push(" | ".dim());
            }
            line.spans.extend(status_line_right.spans);
            Some(line)
        } else {
            Some(status_line_right)
        }
    }

    pub(crate) fn set_status_line_right(&mut self, status_line: Option<Line<'static>>) -> bool {
        if self.footer.status_line_right_value == status_line {
            return false;
        }
        self.footer.status_line_right_value = status_line;
        true
    }
}
