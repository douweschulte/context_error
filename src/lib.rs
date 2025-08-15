//! Contain the definition for errors with all additional data that is needed to generate nice error messages

/// A boxed variant of the error, to ensure a small stack space
mod boxed_error;
/// Wrapping the colored functionality
mod coloured;
/// Helper methods to merge identical errors
mod combine;
/// The context of an error
mod context;
/// An error with all its properties
mod custom_error;
/// A trait to define errors
mod error_trait;
/// A highlight on a line
mod highlight;

pub use boxed_error::*;
use coloured::*;
pub use combine::*;
pub use context::*;
pub use custom_error::*;
pub use error_trait::*;
pub use highlight::*;
