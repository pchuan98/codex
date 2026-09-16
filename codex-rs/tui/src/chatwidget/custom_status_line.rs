use std::path::Component;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use super::ChatWidget;
use crate::bottom_pane::StatusLineItem;
use codex_protocol::config_types::ServiceTier;

mod context;
pub(crate) use context::CustomContextUsage;
mod git_status;
use git_status::DirtyGitStatus;
use git_status::GitStatusCache;

const SEP: &str = " | ";
const GREEN: Color = Color::Green;
const YELLOW: Color = Color::Yellow;
const ORANGE: Color = Color::Rgb(255, 165, 0);
const RED: Color = Color::Red;
const PURPLE: Color = Color::Magenta;
const CYAN: Color = Color::Cyan;
const BLUE: Color = Color::LightBlue;
const DIR: Color = Color::Gray;
const GRAY: Color = Color::DarkGray;
const STREAM_SPEED_REFRESH_INTERVAL: Duration = Duration::from_millis(500);
const STREAM_SPEED_SAMPLE_COUNT: usize = 5;

#[derive(Debug, Default)]
pub(super) struct CustomStatusLineState {
    last_output_tokens_per_second: Option<f64>,
    stream_started_at: Option<Instant>,
    stream_estimated_output_tokens: f64,
    stream_speed_samples: Vec<f64>,
    git_status: GitStatusCache,
    dirty: bool,
    next_refresh_at: Option<Instant>,
}

impl CustomStatusLineState {
    fn record_delta(&mut self, delta: &str, now: Instant) -> bool {
        if delta.is_empty() {
            return false;
        }

        let started_at = *self.stream_started_at.get_or_insert(now);
        self.stream_estimated_output_tokens += estimated_tokens_for_delta(delta);
        let elapsed = now.saturating_duration_since(started_at);
        if elapsed < STREAM_SPEED_REFRESH_INTERVAL {
            return false;
        }

        self.record_speed_sample(elapsed.as_secs_f64());
        self.stream_started_at = Some(now);
        self.stream_estimated_output_tokens = 0.0;
        true
    }

    fn finish_turn(&mut self) {
        if let Some(started_at) = self.stream_started_at {
            let elapsed_secs = Instant::now()
                .saturating_duration_since(started_at)
                .as_secs_f64();
            if elapsed_secs > 0.05 && self.stream_estimated_output_tokens > 0.0 {
                self.record_speed_sample(elapsed_secs);
            }
        }
        self.stream_started_at = None;
        self.stream_estimated_output_tokens = 0.0;
        self.stream_speed_samples.clear();
    }

    fn record_speed_sample(&mut self, elapsed_secs: f64) {
        if elapsed_secs <= 0.0 {
            return;
        }

        self.stream_speed_samples
            .push(self.stream_estimated_output_tokens / elapsed_secs);
        if self.stream_speed_samples.len() > STREAM_SPEED_SAMPLE_COUNT {
            self.stream_speed_samples.remove(0);
        }
        let total: f64 = self.stream_speed_samples.iter().sum();
        self.last_output_tokens_per_second = Some(total / self.stream_speed_samples.len() as f64);
    }
}

impl ChatWidget {
    pub(super) fn refresh_custom_status_line(&mut self) {
        self.custom_status_line.dirty = true;
        self.frame_requester.schedule_frame();
    }

    pub(super) fn refresh_custom_status_line_if_due(&mut self) {
        if self.read_only_history || self.app_event_tx.voice_only.load(Ordering::Relaxed) {
            self.bottom_pane.set_status_line_enabled(/*enabled*/ false);
            self.set_status_line(/*status_line*/ None);
            self.bottom_pane.set_status_line_right(/*status_line*/ None);
            return;
        }
        let now = Instant::now();
        self.custom_status_line.dirty |= self.custom_status_line.git_status.drain_completed();
        if !self.custom_status_line.dirty
            && self
                .custom_status_line
                .next_refresh_at
                .is_some_and(|deadline| now < deadline)
        {
            return;
        }
        self.custom_status_line.dirty = false;
        self.custom_status_line.next_refresh_at = Some(now + git_status::GIT_STATUS_REFRESH_INTERVAL);
        self.bottom_pane.set_status_line_enabled(/*enabled*/ true);
        let line = self.custom_status_line();
        self.set_status_line(Some(line.into()));
        let context = self.custom_context_usage().line();
        self.bottom_pane.set_status_line_right(Some(context));
        self.frame_requester
            .schedule_frame_in(git_status::GIT_STATUS_REFRESH_INTERVAL);
    }

    pub(super) fn record_custom_status_line_delta(&mut self, delta: &str) {
        if self.custom_status_line.record_delta(delta, Instant::now()) {
            self.refresh_custom_status_line();
        }
    }

    pub(super) fn finish_custom_status_line_turn(&mut self) {
        self.custom_status_line.finish_turn();
        self.refresh_custom_status_line();
    }

    fn custom_status_line(&mut self) -> Line<'static> {
        let cwd = self.session_cwd().to_path_buf();
        let git = self.custom_status_line_git_status(&cwd);
        let model = self.status_model_display_name().to_string();
        let reasoning = self.status_line_value_for_item(StatusLineItem::Reasoning);
        let speed_label = (self.current_service_tier() == Some(ServiceTier::Fast.request_value()))
            .then_some("F");

        let mut segments = vec![
            model_context_segment(&model, reasoning.as_deref(), speed_label),
            vec![styled(
                format_dir(
                    &cwd,
                    git.as_ref()
                        .is_some_and(|status| status.repository_root == cwd),
                ),
                DIR,
            )],
        ];
        if let Some(git) = git {
            segments.push(git_segment(&git.branch, git.dirty.as_ref()));
        }
        if let Some(speed) = self.custom_status_line.last_output_tokens_per_second {
            segments.push(vec![styled(format!("{} t/s", format_rate(speed)), CYAN)]);
        }
        join_segments(segments)
    }

    pub(super) fn custom_context_usage(&self) -> CustomContextUsage {
        CustomContextUsage {
            tokens: self.token_info.as_ref().map_or(0, |info| {
                info.last_token_usage.tokens_in_context_window()
            }),
            window: (!self.token_usage_pending)
                .then(|| self.status_line_context_window_size())
                .flatten(),
        }
    }
}

fn model_context_segment(
    model: &str,
    reasoning: Option<&str>,
    speed_label: Option<&str>,
) -> Vec<Span<'static>> {
    let label = reasoning_label(reasoning);
    let mut spans = if let Some(speed_label) = speed_label {
        let color = reasoning_color(reasoning);
        let label = if label == "-" {
            speed_label.to_string()
        } else {
            format!("{label}-{speed_label}")
        };
        vec![styled(label, color)]
    } else {
        vec![styled(label, reasoning_color(reasoning))]
    };
    spans.push(" ".into());
    spans.push(styled(format_model_name(model), BLUE));
    spans
}

fn git_segment(branch: &str, dirty: Option<&DirtyGitStatus>) -> Vec<Span<'static>> {
    let mut spans = vec![styled(branch.to_string(), PURPLE)];
    let Some(dirty) = dirty else {
        return spans;
    };

    if dirty.changed_files > 0 {
        spans.push(" ".into());
        spans.push(styled(format!("~{}", dirty.changed_files), YELLOW));
    }
    let Some((insertions, deletions)) = dirty.lines else {
        spans.push(styled(" +/-?", GRAY));
        return spans;
    };
    if insertions > 0 {
        spans.push(" ".into());
        spans.push(styled(format!("+{insertions}"), GREEN));
    }
    if deletions > 0 {
        spans.push(" ".into());
        spans.push(styled(format!("-{deletions}"), RED));
    }
    spans
}

fn estimated_tokens_for_delta(delta: &str) -> f64 {
    let mut ascii_chars = 0usize;
    let mut cjk_chars = 0usize;
    let mut non_ascii_chars = 0usize;
    for ch in delta.chars() {
        if ch.is_ascii() {
            ascii_chars += 1;
        } else if is_cjk(ch) {
            cjk_chars += 1;
        } else {
            non_ascii_chars += 1;
        }
    }
    (ascii_chars as f64 / 4.0) + (cjk_chars as f64 * 0.75) + non_ascii_chars as f64
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0x2CEB0..=0x2EBEF
            | 0x30000..=0x3134F
            | 0x2F800..=0x2FA1F
    )
}

fn join_segments(segments: Vec<Vec<Span<'static>>>) -> Line<'static> {
    let mut spans = Vec::new();
    for segment in segments {
        if !spans.is_empty() {
            spans.push(styled(SEP.to_string(), GRAY));
        }
        spans.extend(segment);
    }
    Line::from(spans)
}

fn styled(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(color))
}

fn format_model_name(name: &str) -> String {
    let stripped = name.split(" (").next().unwrap_or(name).trim();
    stripped.to_lowercase().replace(' ', "-")
}

fn reasoning_label(level: Option<&str>) -> String {
    match level.unwrap_or_default().to_ascii_lowercase().as_str() {
        "" | "none" | "default" => "-".to_string(),
        "minimal" => "MIN".to_string(),
        "low" => "L".to_string(),
        "medium" => "M".to_string(),
        "high" => "H".to_string(),
        "xhigh" => "XH".to_string(),
        "max" => "MAX".to_string(),
        "ultra" => "ULTRA".to_string(),
        other => other.to_string(),
    }
}

fn reasoning_color(level: Option<&str>) -> Color {
    match level.unwrap_or_default().to_ascii_lowercase().as_str() {
        "low" | "minimal" => GREEN,
        "medium" => YELLOW,
        "high" => ORANGE,
        "xhigh" | "max" | "ultra" => RED,
        _ => GRAY,
    }
}

fn format_dir(path: &Path, is_git_repo_root: bool) -> String {
    if is_git_repo_root && let Some(name) = path_file_name(path) {
        return name;
    }

    match (path_file_name(path), path.parent().and_then(path_file_name)) {
        (Some(name), Some(parent)) => format!("{parent}{}{name}", std::path::MAIN_SEPARATOR),
        (Some(name), None) => name,
        (None, _) => path_root_name(path).unwrap_or_else(|| "-".to_string()),
    }
}

fn path_file_name(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn path_root_name(path: &Path) -> Option<String> {
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                let prefix = prefix.as_os_str().to_string_lossy();
                return Some(prefix.trim_end_matches(':').to_string());
            }
            Component::RootDir => return Some(std::path::MAIN_SEPARATOR.to_string()),
            Component::CurDir | Component::ParentDir | Component::Normal(_) => {}
        }
    }
    None
}

fn format_rate(value: f64) -> String {
    if value >= 1000.0 {
        format!("{:.1}k", value / 1000.0)
    } else {
        format!("{value:.1}")
    }
}

fn heat(t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (start, end, progress) = if t < 0.34 {
        ((0, 255, 0), (255, 255, 0), t / 0.34)
    } else if t < 0.67 {
        ((255, 255, 0), (255, 165, 0), (t - 0.34) / 0.33)
    } else {
        ((255, 165, 0), (255, 0, 0), (t - 0.67) / 0.33)
    };
    let blend = |start: u8, end: u8| {
        (f64::from(start) + (f64::from(end) - f64::from(start)) * progress).round() as u8
    };
    Color::Rgb(
        blend(start.0, end.0),
        blend(start.1, end.1),
        blend(start.2, end.2),
    )
}
