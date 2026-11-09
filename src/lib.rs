pub mod finding;
pub mod lint;
pub mod output;
pub mod scan;

pub use finding::{Finding, Severity, has_high};
pub use scan::lint_path;
