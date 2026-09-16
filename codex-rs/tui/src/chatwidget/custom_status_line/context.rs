//! Context percentage shared by the focused composer and the retained side pane.

use super::GRAY;
use super::heat;
use super::styled;
use crate::token_usage::BASELINE_TOKENS;
use ratatui::text::Line;

/// Context usage retained by the unfocused pane; updated by its own thread notifications.
#[derive(Debug, Default)]
pub(crate) struct CustomContextUsage {
    pub(crate) tokens: i64,
    pub(crate) window: Option<i64>,
}

impl CustomContextUsage {
    pub(crate) fn line(&self) -> Line<'static> {
        let percent = self.window.map(|window| {
            if window <= BASELINE_TOKENS {
                return 100.0;
            }
            let effective_window = window - BASELINE_TOKENS;
            let used = (self.tokens - BASELINE_TOKENS).max(0);
            (used as f64 / effective_window as f64 * 100.0).clamp(0.0, 100.0)
        });
        Line::from(match percent {
            Some(percent) => vec![styled(format!("{percent:.1}%"), heat(percent / 80.0))],
            None => vec![styled("--.-%", GRAY)],
        })
    }
}
