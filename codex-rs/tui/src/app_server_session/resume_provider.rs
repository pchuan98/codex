//! Choose the current provider only for ordinary local cross-provider resumes.

use super::AppServerSession;
use super::ResumeModelSettings;
use super::ThreadParamsMode;
use crate::legacy_core::config::Config;

impl AppServerSession {
    pub(super) async fn local_resume_provider(
        &mut self,
        config: &Config,
        model_settings: ResumeModelSettings,
    ) -> color_eyre::Result<Option<String>> {
        if !matches!(self.thread_params_mode(), ThreadParamsMode::Embedded)
            || !matches!(
                model_settings,
                ResumeModelSettings::RestoreFromThread
                    | ResumeModelSettings::OverrideFromCurrentConfig
            )
        {
            return Ok(None);
        }
        let provider = self
            .history_model_provider(config)
            .await?
            .unwrap_or_else(|| config.model_provider_id.clone());
        Ok(Some(provider))
    }
}
