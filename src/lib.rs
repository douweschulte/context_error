//! Contain the definition for errors with all additional data that is needed to generate nice error messages.
//! ```should_panic
//! use context_error::{CustomError, BasicKind, CreateError, Context};
//! let line = "null,80o0,YES,,67.77";
//! let split = line.split(',').collect::<Vec<&str>>();
//! let num: usize = split[1].parse().map_err(|err| CustomError::new(
//!     BasicKind::Error,
//!     "Invalid number",
//!     "This column is not a number",
//!     Context::default().lines(0, line).add_highlight((0, 5..9))))?;
//! # Ok::<(), CustomError<'static, BasicKind>>(())
//! ```
//! Produces the following error:
//! ```text
//! error: Invalid number
//!  ╷
//!  │ null,80o0,YES,,67.77
//!  ╎      ╶──╴
//!  ╵
//! This column is not a number
//! ```
//!
//! * The properties for an error can be inspected using [StaticErrorContent] and [FullErrorContent].
//! * Errors can be made using [CreateError].
//! * Two error types are already given: [CustomError] and [BoxedError], the latter being a boxed
//!   version to prevent a lot stack space consumed by the result type in the happy case.
//! * Errors can be combined for a more concise error report using [combine_error] and [combine_errors].
//! * Different [ErrorKind]s can be defined to enumerate all possible failure cases for easy matching.
//! * The [Context] for an error can contain a lot of additional details to help highlight exactly
//!   where the error occurred.

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
/// Payload trait for error payloads
mod error_content;
/// A trait to define errors
mod error_create;
/// Trait for error kinds/payloads
mod error_kind;
/// A highlight on a line
mod highlight;

pub use boxed_error::*;
use coloured::*;
pub use combine::*;
pub use context::*;
pub use custom_error::*;
pub use error_content::*;
pub use error_create::*;
pub use error_kind::*;
pub use highlight::*;
