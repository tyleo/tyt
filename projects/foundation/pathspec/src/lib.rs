// Public API

mod error;
mod git_ignore_regex;
mod git_ignore_regex_kind;
mod is_directory_match;
mod is_directory_match_unsigned;
mod is_directory_path_match;
mod is_directory_path_match_unsigned;
mod is_file_match;
mod is_file_match_unsigned;
mod is_file_path_match;
mod is_file_path_match_unsigned;
mod result;
mod unsigned_git_ignore_regex;

pub use error::*;
pub use git_ignore_regex::*;
pub use git_ignore_regex_kind::*;
pub use is_directory_match::*;
pub use is_directory_match_unsigned::*;
pub use is_directory_path_match::*;
pub use is_directory_path_match_unsigned::*;
pub use is_file_match::*;
pub use is_file_match_unsigned::*;
pub use is_file_path_match::*;
pub use is_file_path_match_unsigned::*;
pub use result::*;
pub use unsigned_git_ignore_regex::*;

// Internal API

mod internal;
pub(crate) use internal::*;
