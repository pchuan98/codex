//! Provider selection is independent of the role's permission overrides.

use crate::config::Config;
use crate::session::session::Session;
use crate::thread_manager::build_models_manager;
use codex_config::ConfigLayerEntry;
use codex_config::ConfigLayerSource;
use codex_models_manager::manager::RefreshStrategy;
use codex_models_manager::manager::SharedModelsManager;
use codex_protocol::models::BaseInstructionsProvenance;
use toml::Value as TomlValue;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub(crate) struct ProviderSelectionError(pub(crate) String);

/// Select from the effective provider map without reloading parent CLI overrides.
pub(crate) fn select_provider(config: &mut Config, id: &str) -> Result<(), ProviderSelectionError> {
    let provider = config.model_providers.get(id).cloned().ok_or_else(|| {
        ProviderSelectionError(format!("Model provider `{id}` not found for agent"))
    })?;
    provider.validate().map_err(ProviderSelectionError)?;
    if let Some(key) = &provider.env_key {
        provider.api_key().map_err(|_| {
            ProviderSelectionError(format!(
                "Agent provider `{id}` requires environment variable `{key}`"
            ))
        })?;
    }
    if config.model_provider_id != id || config.model_provider != provider {
        // These overrides describe the parent's model, not the target provider's model.
        let managed_catalog = config.config_layer_stack.requirements().model_catalog_json.is_some()
            || config.config_layer_stack.all_layers_low_to_high().any(|layer| {
                !layer.is_disabled()
                    && layer.name > ConfigLayerSource::SessionFlags
                    && layer.config.get("model_catalog_json").is_some()
            });
        if !managed_catalog {
            config.model_catalog = None;
        }
        config.model_context_window = None;
        config.model_auto_compact_token_limit = None;
        config.prepare_token_budget_for_startup().map_err(|err| {
            ProviderSelectionError(format!("Cannot restore agent model preferences: {err}"))
        })?;
        // A history fork must re-evaluate activation against its own provider/model.
        config.token_budget_startup_config = None;
        if matches!(
            config.base_instructions_provenance,
            Some(BaseInstructionsProvenance::Model { .. })
        ) {
            config.base_instructions = None;
            config.base_instructions_provenance = None;
        }
    }
    config.model_provider_id = id.to_owned();
    config.model_provider = provider;
    Ok(())
}

/// Rebuild model-owned instructions instead of reviving the parent's model prompt from history.
pub(crate) async fn prepare_model_instructions(
    config: &mut Config,
    history: &codex_history::InitialHistory,
    manager: &SharedModelsManager,
) {
    if config.base_instructions.is_none()
        && let Some(instructions) = history.get_base_instructions()
        && matches!(
            instructions.provenance,
            Some(BaseInstructionsProvenance::Model { .. })
        )
        && let Some(model) = config.model.as_ref()
    {
        let info = manager
            .get_model_info(model, &config.to_models_manager_config())
            .await;
        config.base_instructions = Some(info.get_model_instructions(config.personality));
        config.base_instructions_provenance = Some(BaseInstructionsProvenance::Model {
            model: info.slug,
        });
    }
}

/// Keep the projected TOML consistent with whole-provider replacement in typed config.
pub(crate) fn project_role_layers(
    layers: &mut [ConfigLayerEntry],
    role: &TomlValue,
    parent: &Config,
    child: &Config,
) {
    let built_ins = codex_model_provider_info::built_in_model_providers(/*openai_base_url*/ None);
    let connection_changed = parent.model_provider_id != child.model_provider_id
        || parent.model_provider != child.model_provider;
    for layer in layers {
        if layer.is_disabled() || layer.name > ConfigLayerSource::SessionFlags {
            continue;
        }
        let mut projected = layer.config.clone();
        if let Some(definitions) = role.get("model_providers").and_then(TomlValue::as_table)
            && let Some(providers) = projected
                .get_mut("model_providers")
                .and_then(TomlValue::as_table_mut)
        {
            for id in definitions.keys().filter(|id| !built_ins.contains_key(*id)) {
                providers.remove(id);
            }
        }
        if connection_changed
            && let Some(table) = projected.as_table_mut()
        {
            for (key, cleared) in [
                ("model_catalog_json", child.model_catalog.is_none()),
                ("model_context_window", child.model_context_window.is_none()),
                (
                    "model_auto_compact_token_limit",
                    child.model_auto_compact_token_limit.is_none(),
                ),
            ] {
                if cleared {
                    table.remove(key);
                }
            }
        }
        if projected != layer.config {
            layer.version = ConfigLayerEntry::new(layer.name.clone(), projected.clone()).version;
            layer.config = projected;
        }
    }
}

/// Share a catalog only when both the connection and explicit catalog are unchanged.
pub(crate) async fn models_manager_for_config(
    session: &Session,
    config: &Config,
) -> SharedModelsManager {
    let parent = session.get_config().await;
    if parent.model_provider_id == config.model_provider_id
        && parent.model_provider == config.model_provider
        && parent.model_catalog == config.model_catalog
    {
        return session.services.models_manager.clone();
    }
    let manager = build_models_manager(config, session.services.auth_manager.clone());
    manager
        .list_models(RefreshStrategy::OnlineIfUncached, config.http_client_factory())
        .await;
    manager
}
