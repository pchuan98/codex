//! Local resume routing; the selected route still passes upstream policy validation.

use super::ConfigManager;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ResumeProviderPolicy {
    Saved,
    CurrentOnExplicitOverride,
}

impl ConfigManager {
    pub(crate) fn with_local_resume_provider(mut self) -> Self {
        self.resume_provider_policy = ResumeProviderPolicy::CurrentOnExplicitOverride;
        self
    }

    pub(super) async fn resolve_resume_provider(
        &self,
        saved: Option<&str>,
        requested: Option<&str>,
        cwd: Option<PathBuf>,
    ) -> std::io::Result<Option<String>> {
        if self.resume_provider_policy == ResumeProviderPolicy::CurrentOnExplicitOverride
            && let (Some(saved), Some(requested)) = (saved, requested)
            && saved != requested
        {
            let current = self.load_latest_config(cwd).await?;
            if current.model_provider_id == requested {
                return Ok(Some(requested.to_owned()));
            }
        }
        Ok(saved.map(str::to_owned))
    }
}
