use std::collections::HashMap;
use std::collections::VecDeque;

use super::*;

#[derive(Debug, Default)]
pub(super) struct ThreadTokenUsageState {
    by_thread: HashMap<ThreadId, TokenUsageInfo>,
    recency: VecDeque<ThreadId>,
}

const MAX_CACHED_THREADS: usize = 64;

impl ThreadTokenUsageState {
    fn insert(&mut self, thread_id: ThreadId, info: TokenUsageInfo) {
        self.recency.retain(|cached| *cached != thread_id);
        self.recency.push_back(thread_id);
        self.by_thread.insert(thread_id, info);
        while self.recency.len() > MAX_CACHED_THREADS {
            if let Some(expired) = self.recency.pop_front() {
                self.by_thread.remove(&expired);
            }
        }
    }

    fn remove(&mut self, thread_id: ThreadId) {
        self.by_thread.remove(&thread_id);
        self.recency.retain(|cached| *cached != thread_id);
    }
}

impl ChatWidget {
    #[cfg(test)]
    pub(crate) fn set_token_info(&mut self, info: Option<TokenUsageInfo>) {
        if let Some(thread_id) = self.thread_id {
            match &info {
                Some(info) => {
                    self.thread_token_usage.insert(thread_id, info.clone());
                }
                None => {
                    self.thread_token_usage.remove(thread_id);
                }
            }
        }
        self.apply_token_info_option(info);
    }

    pub(crate) fn set_thread_token_info(&mut self, thread_id: ThreadId, info: TokenUsageInfo) {
        self.thread_token_usage.insert(thread_id, info.clone());
        if self.thread_id == Some(thread_id) {
            self.apply_token_info(info);
        }
    }

    pub(super) fn restore_current_thread_token_info(&mut self) {
        let info = self
            .thread_id
            .and_then(|thread_id| self.thread_token_usage.by_thread.get(&thread_id).cloned());
        self.apply_token_info_option(info);
    }

    fn apply_token_info_option(&mut self, info: Option<TokenUsageInfo>) {
        match info {
            Some(info) => self.apply_token_info(info),
            None => {
                self.bottom_pane
                    .set_context_window(/*percent*/ None, /*used_tokens*/ None);
                self.token_info = None;
            }
        }
    }

    pub(crate) fn clear_token_usage(&mut self) {
        if let Some(thread_id) = self.thread_id {
            self.thread_token_usage.remove(thread_id);
        }
        self.token_info = None;
        self.bottom_pane
            .set_context_window(/*percent*/ None, /*used_tokens*/ None);
    }
}
