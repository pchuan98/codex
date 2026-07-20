use std::path::Component;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use chrono::DateTime;
use chrono::Duration as ChronoDuration;
use chrono::Local;
use chrono::Utc;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use super::rate_limits::get_limits_duration;
use super::*;
use crate::bottom_pane::StatusLineItem;
use crate::status::RATE_LIMIT_STALE_THRESHOLD_MINUTES;
use codex_protocol::config_types::ServiceTier;

mod git_status;
mod visibility;
use git_status::DirtyGitStatus;
pub(crate) use git_status::CustomStatusLineGitStatus;
use git_status::GitStatusCache;

const SEP: &str = " | ";
const GREEN: Color = Color::Green;
const YELLOW: Color = Color::Yellow;
const ORANGE: Color = Color::LightYellow;
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
    rate_limit: Option<CustomRateLimitSnapshot>,
}

#[derive(Debug, Clone)]
struct CustomRateLimitSnapshot {
    captured_at: DateTime<Local>,
    primary: Option<CustomRateLimitWindow>,
    secondary: Option<CustomRateLimitWindow>,
}

#[derive(Debug, Clone)]
struct CustomRateLimitWindow {
    used_percent: f64,
    window_minutes: Option<i64>,
    resets_at: Option<DateTime<Local>>,
}

struct CustomStatusLine {
    line: Option<Line<'static>>,
    next_countdown_refresh: Option<Duration>,
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

    fn update_rate_limit(&mut self, snapshot: &RateLimitSnapshot, captured_at: DateTime<Local>) {
        let limit_id = snapshot.limit_id.as_deref().unwrap_or("codex");
        if !limit_id.eq_ignore_ascii_case("codex") {
            return;
        }
        self.rate_limit = Some(CustomRateLimitSnapshot {
            captured_at,
            primary: snapshot
                .primary
                .as_ref()
                .map(CustomRateLimitWindow::from_protocol),
            secondary: snapshot
                .secondary
                .as_ref()
                .map(CustomRateLimitWindow::from_protocol),
        });
    }
}

impl CustomRateLimitWindow {
    fn from_protocol(window: &codex_app_server_protocol::RateLimitWindow) -> Self {
        Self {
            used_percent: f64::from(window.used_percent),
            window_minutes: window.window_duration_mins,
            resets_at: window
                .resets_at
                .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp, 0))
                .map(|timestamp| timestamp.with_timezone(&Local)),
        }
    }
}

impl ChatWidget {
    pub(super) fn refresh_custom_status_line(&mut self) {
        if !visibility::custom_status_line_enabled() {
            self.bottom_pane.set_status_line_enabled(/*enabled*/ false);
            self.set_status_line(None);
            self.set_status_line_right(None);
            self.set_status_line_hyperlink(/*url*/ None);
            return;
        }

        self.bottom_pane.set_status_line_enabled(/*enabled*/ true);
        let status_line = self.custom_status_line();
        if status_line.next_countdown_refresh.is_some() {
            self.frame_requester
                .schedule_frame_in(Duration::from_secs(60));
        }
        self.set_status_line(status_line.line);
        self.set_status_line_right(Some(context_used_percent_line(
            self.context_used_percent_precise(),
        )));
        self.set_status_line_hyperlink(/*url*/ None);
    }

    pub(super) fn sync_custom_status_line_rate_limit(
        &mut self,
        snapshot: Option<&RateLimitSnapshot>,
        captured_at: DateTime<Local>,
    ) {
        match snapshot {
            Some(snapshot) => self.custom_status_line.update_rate_limit(snapshot, captured_at),
            None => self.custom_status_line.rate_limit = None,
        }
    }

    pub(super) fn record_custom_status_line_delta(&mut self, delta: &str) {
        if !self.custom_status_line_active() {
            return;
        }

        if self.custom_status_line.record_delta(delta, Instant::now()) {
            self.refresh_custom_status_line();
            self.frame_requester
                .schedule_frame_in(STREAM_SPEED_REFRESH_INTERVAL);
        }
    }

    pub(super) fn finish_custom_status_line_turn(&mut self) {
        self.custom_status_line.finish_turn();
        if self.custom_status_line_active() {
            self.refresh_custom_status_line();
        }
    }

    fn custom_status_line_active(&self) -> bool {
        visibility::custom_status_line_enabled()
    }

    fn custom_status_line(&mut self) -> CustomStatusLine {
        let cwd = self
            .current_cwd
            .as_deref()
            .unwrap_or(self.config.cwd.as_path())
            .to_path_buf();
        let git = self.custom_status_line_git_status(&cwd);
        let rate_limits = self
            .custom_status_line
            .rate_limit
            .as_ref()
            .and_then(|snapshot| rate_limit_segment(snapshot, Local::now()));
        let model = self.model_display_name().to_string();
        let reasoning = self.status_line_value_for_item(StatusLineItem::Reasoning);
        let fast_mode = self.current_service_tier() == Some(ServiceTier::Fast.request_value());

        let mut segments = vec![
            model_context_segment(&model, reasoning.as_deref(), fast_mode),
            vec![styled(
                format_dir(
                    &cwd,
                    git.as_ref()
                        .is_some_and(|status| status.cwd_is_repository_root),
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
        if let Some((segment, next_countdown_refresh)) = rate_limits {
            segments.push(segment);
            return CustomStatusLine {
                line: Some(join_segments(segments)),
                next_countdown_refresh,
            };
        }

        CustomStatusLine {
            line: Some(join_segments(segments)),
            next_countdown_refresh: None,
        }
    }

    fn context_used_percent_precise(&self) -> Option<f64> {
        let window = self.status_line_context_window_size()?;
        let default_usage = TokenUsage::default();
        let usage = self
            .token_info
            .as_ref()
            .map(|info| &info.last_token_usage)
            .unwrap_or(&default_usage);
        Some(usage.percent_of_context_window_used_precise(window))
    }
}

fn model_context_segment(
    model: &str,
    reasoning: Option<&str>,
    fast_mode: bool,
) -> Vec<Span<'static>> {
    let label = reasoning_label(reasoning);
    let mut spans = if fast_mode {
        let color = reasoning_color(reasoning);
        let label = if label == "-" {
            "F".to_string()
        } else {
            format!("{label}-F")
        };
        vec![styled(label, color)]
    } else {
        vec![styled(label, reasoning_color(reasoning))]
    };
    spans.push(" ".into());
    spans.push(styled(format_model_name(model), BLUE));
    spans
}

fn context_used_percent_line(percent: Option<f64>) -> Line<'static> {
    Line::from(match percent {
        Some(percent) => vec![styled(format!("{percent:.1}%"), heat(percent / 80.0))],
        None => vec![styled("--.-%", GRAY)],
    })
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
    if dirty.insertions > 0 {
        spans.push(" ".into());
        spans.push(styled(format!("+{}", dirty.insertions), GREEN));
    }
    if dirty.deletions > 0 {
        spans.push(" ".into());
        spans.push(styled(format!("-{}", dirty.deletions), RED));
    }
    spans
}

fn rate_limit_segment(
    snapshot: &CustomRateLimitSnapshot,
    now: DateTime<Local>,
) -> Option<(Vec<Span<'static>>, Option<Duration>)> {
    if now.signed_duration_since(snapshot.captured_at)
        > ChronoDuration::minutes(RATE_LIMIT_STALE_THRESHOLD_MINUTES)
    {
        return None;
    }
    let five_hour = five_hour_window(snapshot);
    let weekly = weekly_window(snapshot);
    if five_hour.is_none() && weekly.is_none() {
        return None;
    }

    let weekly_only = five_hour.is_none();
    let countdown = if weekly_only {
        weekly.and_then(|window| countdown_until_reset(window, now))
    } else {
        five_hour.and_then(|window| countdown_until_reset(window, now))
    };
    let mut spans = if weekly_only {
        vec![percent_span(weekly.map(|window| window.used_percent))]
    } else {
        vec![
            percent_span(five_hour.map(|window| window.used_percent)),
            " - ".into(),
            percent_span(weekly.map(|window| window.used_percent)),
        ]
    };
    if let Some(countdown) = countdown {
        spans.push(styled(" ".to_string(), GRAY));
        spans.push(styled(
            format_countdown(countdown),
            heat(reset_heat(countdown)),
        ));
    }
    // Refresh once per minute even when the backend omitted a reset timestamp so stale snapshots
    // disappear without requiring another unrelated UI event.
    Some((spans, Some(Duration::from_secs(60))))
}

fn five_hour_window(snapshot: &CustomRateLimitSnapshot) -> Option<&CustomRateLimitWindow> {
    find_window(snapshot, "5h")
        .or_else(|| non_weekly_primary_window(snapshot))
        .or_else(|| non_weekly_secondary_window_when_primary_is_weekly(snapshot))
}

fn weekly_window(snapshot: &CustomRateLimitSnapshot) -> Option<&CustomRateLimitWindow> {
    find_window(snapshot, "weekly").or(snapshot.secondary.as_ref())
}

fn find_window<'a>(
    snapshot: &'a CustomRateLimitSnapshot,
    label: &str,
) -> Option<&'a CustomRateLimitWindow> {
    snapshot
        .primary
        .as_ref()
        .filter(|window| matches_window_label(window, label))
        .or_else(|| {
            snapshot
                .secondary
                .as_ref()
                .filter(|window| matches_window_label(window, label))
        })
}

fn non_weekly_primary_window(
    snapshot: &CustomRateLimitSnapshot,
) -> Option<&CustomRateLimitWindow> {
    snapshot
        .primary
        .as_ref()
        .filter(|window| !matches_window_label(window, "weekly"))
}

fn non_weekly_secondary_window_when_primary_is_weekly(
    snapshot: &CustomRateLimitSnapshot,
) -> Option<&CustomRateLimitWindow> {
    let primary = snapshot.primary.as_ref()?;
    if !matches_window_label(primary, "weekly") {
        return None;
    }
    snapshot
        .secondary
        .as_ref()
        .filter(|window| !matches_window_label(window, "weekly"))
}

fn matches_window_label(window: &CustomRateLimitWindow, label: &str) -> bool {
    window
        .window_minutes
        .and_then(get_limits_duration)
        .as_deref()
        == Some(label)
}

fn countdown_until_reset(
    window: &CustomRateLimitWindow,
    now: DateTime<Local>,
) -> Option<Duration> {
    window
        .resets_at?
        .signed_duration_since(now)
        .to_std()
        .ok()
}

fn percent_span(percent: Option<f64>) -> Span<'static> {
    match percent {
        Some(percent) => styled(format!("{}%", percent.round() as i64), heat(percent / 80.0)),
        None => styled("--%", GRAY),
    }
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

fn format_countdown(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
    if total_secs > SECONDS_PER_DAY {
        let days = total_secs.div_ceil(SECONDS_PER_DAY);
        return format!("{days}d");
    }
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    format!("{hours}:{minutes:02}")
}

fn reset_heat(duration: Duration) -> f64 {
    1.0 - (duration.as_secs_f64() / (5.0 * 3600.0))
}

fn heat(t: f64) -> Color {
    match t.clamp(0.0, 1.0) {
        t if t < 0.34 => GREEN,
        t if t < 0.67 => YELLOW,
        t if t < 1.0 => ORANGE,
        _ => RED,
    }
}
