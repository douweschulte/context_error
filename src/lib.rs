//! Contain the definition for errors with all additional data that is needed to generate nice error messages

/// The context of an error
mod context;
/// An error with all its properties
mod custom_error;
/// A highlight on a line
mod highlight;

pub use context::*;
pub use custom_error::*;
pub use highlight::*;
