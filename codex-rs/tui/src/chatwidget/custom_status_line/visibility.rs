//! Local kill switch for the customized footer statusline.
//!
//! Keep this as a tiny separate hook so testing or rebasing the statusline
//! customization only needs a one-line change here.

pub(super) fn custom_status_line_enabled() -> bool {
    true
}
