use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use super::*;
use crate::workspace_command::WorkspaceCommand;
use crate::workspace_command::WorkspaceCommandExecutor;
use tokio::sync::oneshot;

pub(super) const GIT_STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const GIT_STATUS_COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_CACHED_REPOSITORIES: usize = 16;
const MAX_CACHED_DIRECTORIES: usize = 64;
// Stay below command/exec's default 1 MiB cap: a capped prefix must never look complete.
const MAX_GIT_OUTPUT_BYTES: usize = 512 * 1024;

#[derive(Debug, Default)]
pub(super) struct GitStatusCache {
    by_repository: BTreeMap<PathBuf, GitStatusCacheEntry>,
    repository_by_cwd: VecDeque<(PathBuf, PathBuf)>,
}

#[derive(Debug, Default)]
struct GitStatusCacheEntry {
    status: Option<CustomStatusLineGitStatus>,
    pending: Option<GitStatusRequest>,
    last_requested_at: Option<Instant>,
}

#[derive(Debug)]
struct GitStatusRequest {
    cwd: PathBuf,
    result: oneshot::Receiver<Option<CustomStatusLineGitStatus>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CustomStatusLineGitStatus {
    pub(super) repository_root: PathBuf,
    pub(super) branch: String,
    pub(super) dirty: Option<DirtyGitStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DirtyGitStatus {
    pub(super) changed_files: usize,
    pub(super) lines: Option<(u64, u64)>,
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
            .iter()
            .find(|(directory, _)| directory == cwd)
            .map(|(_, repository)| repository.clone())
            .unwrap_or_else(|| cwd.to_path_buf());
        self.request_custom_status_line_git_status(cwd.to_path_buf(), cache_key.clone());
        self.custom_status_line
            .git_status
            .by_repository
            .get(&cache_key)
            .and_then(|entry| entry.status.clone())
    }

    fn request_custom_status_line_git_status(&mut self, cwd: PathBuf, cache_key: PathBuf) {
        let now = Instant::now();
        let cache = &mut self.custom_status_line.git_status;
        if !cache.by_repository.contains_key(&cache_key)
            && cache.by_repository.len() >= MAX_CACHED_REPOSITORIES
            && !cache.evict_oldest_repository()
        {
            return;
        }
        let entry = cache.by_repository.entry(cache_key).or_default();
        if entry.pending.is_some()
            || entry
                .last_requested_at
                .is_some_and(|last| now.saturating_duration_since(last) < GIT_STATUS_REFRESH_INTERVAL)
        {
            return;
        }

        entry.last_requested_at = Some(now);
        let Some(runner) = self.workspace_command_runner.clone() else {
            return;
        };
        let (tx, rx) = oneshot::channel();
        entry.pending = Some(GitStatusRequest {
            cwd: cwd.clone(),
            result: rx,
        });
        let frame_requester = self.frame_requester.clone();
        tokio::spawn(async move {
            let status = load_git_status(runner.as_ref(), &cwd).await;
            if tx.send(status).is_ok() {
                frame_requester.schedule_frame();
            }
        });
    }
}

impl GitStatusCache {
    // Each result receiver stays with the widget/backend that started the query. Moving focus
    // cannot deliver another pane's Git result or leave the original pane permanently pending.
    pub(super) fn drain_completed(&mut self) -> bool {
        let completed: Vec<_> = self
            .by_repository
            .iter_mut()
            .filter_map(|(key, entry)| {
                let request = entry.pending.as_mut()?;
                let status = match request.result.try_recv() {
                    Ok(status) => status,
                    Err(oneshot::error::TryRecvError::Empty) => return None,
                    Err(oneshot::error::TryRecvError::Closed) => None,
                };
                let cwd = request.cwd.clone();
                entry.pending = None;
                Some((key.clone(), cwd, status))
            })
            .collect();
        let changed = !completed.is_empty();
        for (key, cwd, status) in completed {
            let Some(status) = status else {
                continue;
            };
            let repository_root = status.repository_root.clone();
            let Some(mut entry) = self.by_repository.remove(&key) else {
                continue;
            };
            self.repository_by_cwd.retain(|(directory, _)| directory != &cwd);
            self.repository_by_cwd.push_back((cwd, repository_root.clone()));
            if self.repository_by_cwd.len() > MAX_CACHED_DIRECTORIES {
                self.repository_by_cwd.pop_front();
            }
            if let Some(existing) = self.by_repository.get_mut(&repository_root) {
                existing.status = Some(status);
                existing.last_requested_at = existing.last_requested_at.max(entry.last_requested_at);
            } else {
                entry.status = Some(status);
                self.by_repository.insert(repository_root, entry);
            }
        }
        changed
    }

    // Never discard a live receiver: at most one task per entry and 16 entries per widget.
    fn evict_oldest_repository(&mut self) -> bool {
        let Some(oldest) = self
            .by_repository
            .iter()
            .filter(|(_, entry)| entry.pending.is_none())
            .min_by_key(|(_, entry)| entry.last_requested_at)
            .map(|(path, _)| path.clone())
        else {
            return false;
        };
        self.by_repository.remove(&oldest);
        self.repository_by_cwd
            .retain(|(_, repository)| repository != &oldest);
        true
    }
}

async fn load_git_status(
    runner: &dyn WorkspaceCommandExecutor,
    cwd: &Path,
) -> Option<CustomStatusLineGitStatus> {
    let (repository, porcelain) = tokio::join!(
        git_output(runner, cwd, &["rev-parse", "--show-toplevel"]),
        git_output(
            runner,
            cwd,
            &["status", "--porcelain=v2", "--branch", "--untracked-files=all"],
        ),
    );
    let porcelain = porcelain?;
    let branch = parse_branch(&porcelain)?;
    let repository = repository?;
    let repository_root = PathBuf::from(repository.lines().next().filter(|line| !line.is_empty())?);
    let dirty = dirty_git_status(runner, cwd, &porcelain).await.ok()?;
    Some(CustomStatusLineGitStatus {
        repository_root,
        branch,
        dirty,
    })
}

fn parse_branch(porcelain: &str) -> Option<String> {
    let branch = porcelain
        .lines()
        .find_map(|line| line.strip_prefix("# branch.head "))?
        .trim();
    if branch == "(detached)" {
        let commit = porcelain
            .lines()
            .find_map(|line| line.strip_prefix("# branch.oid "));
        return Some(commit.map_or_else(
            || "detached".to_string(),
            |oid| format!("detached {}", oid.chars().take(7).collect::<String>()),
        ));
    }
    (!branch.is_empty()).then(|| branch.to_string())
}

async fn dirty_git_status(
    runner: &dyn WorkspaceCommandExecutor,
    cwd: &Path,
    porcelain: &str,
) -> Result<Option<DirtyGitStatus>, ()> {
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
        return Ok(None);
    }

    // An unborn branch has no HEAD baseline. Keep its known file count, not fake zero lines.
    let lines = if porcelain
        .lines()
        .any(|line| line == "# branch.oid (initial)")
    {
        None
    } else {
        let stdout = git_output(runner, cwd, &["diff", "HEAD", "--numstat"])
            .await
            .ok_or(())?;
        Some(parse_numstat(&stdout).ok_or(())?)
    };
    Ok(Some(DirtyGitStatus {
        changed_files,
        lines,
    }))
}

async fn git_output(
    runner: &dyn WorkspaceCommandExecutor,
    cwd: &Path,
    args: &[&str],
) -> Option<String> {
    let mut argv = Vec::with_capacity(args.len() + 3);
    argv.extend(["git", "-c", "core.quotePath=true"].map(str::to_string));
    argv.extend(args.iter().map(|arg| (*arg).to_string()));
    let output = runner
        .run(
            WorkspaceCommand::new(argv, cwd.to_path_buf())
                .env("GIT_OPTIONAL_LOCKS", "0")
                .env("GIT_TERMINAL_PROMPT", "0")
                .timeout(GIT_STATUS_COMMAND_TIMEOUT),
        )
        .await
        .ok()?;
    (output.success()
        && output.stdout.len() < MAX_GIT_OUTPUT_BYTES
        && output.stderr.len() < MAX_GIT_OUTPUT_BYTES)
        .then_some(output.stdout)
}

fn parse_numstat(stdout: &str) -> Option<(u64, u64)> {
    stdout
        .lines()
        .try_fold((0u64, 0u64), |(insertions, deletions), line| {
            let mut columns = line.splitn(3, '\t');
            let added = columns.next()?;
            let removed = columns.next()?;
            columns.next()?;
            if added == "-" && removed == "-" {
                return Some((insertions, deletions));
            }
            Some((
                insertions.checked_add(added.parse::<u64>().ok()?)?,
                deletions.checked_add(removed.parse::<u64>().ok()?)?,
            ))
        })
}
