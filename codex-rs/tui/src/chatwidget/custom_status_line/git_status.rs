use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use super::*;
use crate::app_event::AppEvent;
use crate::workspace_command::WorkspaceCommand;
use crate::workspace_command::WorkspaceCommandExecutor;

const GIT_STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const GIT_STATUS_COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_CACHED_REPOSITORIES: usize = 16;

#[derive(Debug, Default)]
pub(super) struct GitStatusCache {
    by_repository: BTreeMap<PathBuf, GitStatusCacheEntry>,
    repository_by_cwd: BTreeMap<PathBuf, PathBuf>,
}

#[derive(Clone, Debug, Default)]
struct GitStatusCacheEntry {
    status: Option<CustomStatusLineGitStatus>,
    pending: bool,
    last_requested_at: Option<Instant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CustomStatusLineGitStatus {
    repository_root: PathBuf,
    pub(super) cwd_is_repository_root: bool,
    pub(super) branch: String,
    pub(super) dirty: Option<DirtyGitStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DirtyGitStatus {
    pub(super) changed_files: usize,
    pub(super) insertions: u64,
    pub(super) deletions: u64,
}

impl ChatWidget {
    pub(super) fn custom_status_line_git_status(
        &mut self,
        cwd: &Path,
    ) -> Option<CustomStatusLineGitStatus> {
        let cache_key = self
            .custom_status_line
            .git_status
            .repository_by_cwd
            .get(cwd)
            .cloned()
            .unwrap_or_else(|| cwd.to_path_buf());
        self.request_custom_status_line_git_status(cwd.to_path_buf(), cache_key.clone());
        self.custom_status_line
            .git_status
            .by_repository
            .get(&cache_key)
            .and_then(|entry| entry.status.clone())
    }

    pub(crate) fn finish_custom_status_line_git_status_refresh(
        &mut self,
        cwd: PathBuf,
        status: Option<CustomStatusLineGitStatus>,
    ) -> bool {
        let cache = &mut self.custom_status_line.git_status;
        let previous_repository = cache.repository_by_cwd.get(&cwd).cloned();
        let previous_key = previous_repository
            .clone()
            .unwrap_or_else(|| cwd.clone());
        let requested_at = cache
            .by_repository
            .get_mut(&previous_key)
            .and_then(|entry| {
                entry.pending = false;
                entry.last_requested_at
            });

        if let Some(status) = status {
            let repository_root = status.repository_root.clone();
            if previous_repository.is_none() && previous_key != repository_root {
                cache.by_repository.remove(&previous_key);
            }
            cache
                .repository_by_cwd
                .insert(cwd, repository_root.clone());
            let entry = cache.by_repository.entry(repository_root).or_default();
            entry.status = Some(status);
            entry.pending = false;
            entry.last_requested_at = requested_at.or(entry.last_requested_at);
        } else if !cache.by_repository.contains_key(&previous_key) {
            cache.by_repository.insert(
                previous_key,
                GitStatusCacheEntry {
                    last_requested_at: requested_at,
                    ..Default::default()
                },
            );
        }
        cache.evict_oldest_repositories();
        self.refresh_custom_status_line();
        true
    }

    fn request_custom_status_line_git_status(&mut self, cwd: PathBuf, cache_key: PathBuf) {
        let now = Instant::now();
        let cache = &mut self.custom_status_line.git_status;
        cache.evict_oldest_repository_before_inserting(&cache_key);
        let entry = self
            .custom_status_line
            .git_status
            .by_repository
            .entry(cache_key)
            .or_default();
        if entry.pending
            || entry
                .last_requested_at
                .is_some_and(|last| now.saturating_duration_since(last) < GIT_STATUS_REFRESH_INTERVAL)
        {
            return;
        }

        entry.pending = true;
        entry.last_requested_at = Some(now);
        let Some(runner) = self.workspace_command_runner.clone() else {
            entry.pending = false;
            return;
        };
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let status = load_git_status(runner.as_ref(), &cwd).await;
            tx.send(AppEvent::CustomStatusLineGitStatusUpdated { cwd, status });
        });
    }
}

impl GitStatusCache {
    fn evict_oldest_repository_before_inserting(&mut self, cache_key: &Path) {
        if !self.by_repository.contains_key(cache_key)
            && self.by_repository.len() >= MAX_CACHED_REPOSITORIES
        {
            self.evict_oldest_repository();
        }
    }

    fn evict_oldest_repositories(&mut self) {
        while self.by_repository.len() > MAX_CACHED_REPOSITORIES {
            self.evict_oldest_repository();
        }
    }

    fn evict_oldest_repository(&mut self) {
        let Some(oldest) = self
            .by_repository
            .iter()
            .min_by_key(|(_, entry)| entry.last_requested_at)
            .map(|(path, _)| path.clone())
        else {
            return;
        };
        self.by_repository.remove(&oldest);
        self.repository_by_cwd
            .retain(|_, repository| repository != &oldest);
    }
}

async fn load_git_status(
    runner: &dyn WorkspaceCommandExecutor,
    cwd: &Path,
) -> Option<CustomStatusLineGitStatus> {
    let (repository, porcelain) = tokio::join!(
        git_output(runner, cwd, &["rev-parse", "--show-toplevel", "--show-prefix"]),
        git_output(runner, cwd, &["status", "--porcelain=v2", "--branch"]),
    );
    let (repository_root, cwd_is_repository_root) = parse_repository(repository?.as_str())?;
    let porcelain = porcelain?;
    let branch = parse_branch(&porcelain)?;
    let dirty = dirty_git_status(runner, cwd, &porcelain).await;
    Some(CustomStatusLineGitStatus {
        repository_root,
        cwd_is_repository_root,
        branch,
        dirty,
    })
}

fn parse_repository(stdout: &str) -> Option<(PathBuf, bool)> {
    let mut lines = stdout.split('\n').map(|line| line.trim_end_matches('\r'));
    let repository_root = PathBuf::from(lines.next()?);
    if repository_root.as_os_str().is_empty() {
        return None;
    }
    let prefix = lines.next().unwrap_or_default();
    Some((repository_root, prefix.is_empty()))
}

fn parse_branch(porcelain: &str) -> Option<String> {
    porcelain.lines().find_map(|line| {
        let branch = line.strip_prefix("# branch.head ")?.trim();
        (!branch.is_empty() && branch != "(detached)").then(|| branch.to_string())
    })
}

async fn dirty_git_status(
    runner: &dyn WorkspaceCommandExecutor,
    cwd: &Path,
    porcelain: &str,
) -> Option<DirtyGitStatus> {
    let changed_files = porcelain
        .lines()
        .filter(|line| {
            line.starts_with("1 ")
                || line.starts_with("2 ")
                || line.starts_with("u ")
                || line.starts_with("? ")
        })
        .count();
    if changed_files == 0 {
        return None;
    }

    let (insertions, deletions) = git_output(runner, cwd, &["diff", "HEAD", "--numstat"])
        .await
        .map(|stdout| parse_numstat(&stdout))
        .unwrap_or((0, 0));
    Some(DirtyGitStatus {
        changed_files,
        insertions,
        deletions,
    })
}

async fn git_output(
    runner: &dyn WorkspaceCommandExecutor,
    cwd: &Path,
    args: &[&str],
) -> Option<String> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("git".to_string());
    argv.extend(args.iter().map(|arg| (*arg).to_string()));
    let output = runner
        .run(
            WorkspaceCommand::new(argv)
                .cwd(cwd.to_path_buf())
                .env("GIT_OPTIONAL_LOCKS", "0")
                .env("GIT_TERMINAL_PROMPT", "0")
                .timeout(GIT_STATUS_COMMAND_TIMEOUT),
        )
        .await
        .ok()?;
    output.success().then_some(output.stdout)
}

fn parse_numstat(stdout: &str) -> (u64, u64) {
    stdout
        .lines()
        .fold((0, 0), |(insertions, deletions), line| {
            let mut columns = line.split('\t');
            (
                insertions
                    + columns
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(0),
                deletions
                    + columns
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(0),
            )
        })
}
