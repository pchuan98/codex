//! Retain one usage snapshot per thread independently of the bounded transcript event buffer.

use super::ThreadBufferedEvent;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadTokenUsageUpdatedNotification;

#[derive(Debug, Default)]
pub(super) struct ThreadContextUsage {
    latest: Option<ThreadTokenUsageUpdatedNotification>,
}

impl ThreadContextUsage {
    pub(super) fn observe(&mut self, notification: &ServerNotification) -> bool {
        match notification {
            ServerNotification::ThreadTokenUsageUpdated(usage) => {
                self.latest = Some(usage.clone());
                true
            }
            ServerNotification::ThreadReverted(_) => {
                self.latest = None;
                false
            }
            _ => false,
        }
    }

    pub(super) fn append_to(&self, events: &mut Vec<ThreadBufferedEvent>) {
        if let Some(usage) = &self.latest {
            events.push(ThreadBufferedEvent::Notification(Box::new(
                ServerNotification::ThreadTokenUsageUpdated(usage.clone()),
            )));
        }
    }
}
