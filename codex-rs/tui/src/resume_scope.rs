use codex_app_server_protocol::ThreadSourceKind;

pub(crate) fn main_session_source_kinds() -> Vec<ThreadSourceKind> {
    crate::resume_source_kinds(/*include_non_interactive*/ true)
}
