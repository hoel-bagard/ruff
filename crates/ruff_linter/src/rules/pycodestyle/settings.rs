//! Settings for the `pycodestyle` plugin.

use ruff_macros::CacheKey;

use crate::line_width::LineLength;

#[derive(Debug, Default, CacheKey)]
pub struct Settings {
    pub max_line_length: LineLength,
    pub max_doc_length: Option<LineLength>,
    pub ignore_overlong_task_comments: bool,
    pub hang_closing: bool,
}
