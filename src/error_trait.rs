use std::borrow::Cow;

use crate::{Context, CustomError};

/// A trait to guarantee identical an API between the boxed and unboxed error version
pub trait CustomErrorTrait<'text>: Sized + Default {
    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `short_desc` - A short description of the error, used as title line.
    /// * `long_desc` - A longer description of the error, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    fn error(
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self;

    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `short_desc` - A short description of the warning, generally used as title line.
    /// * `long_desc` - A longer description of the warning, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    fn warning(
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self;

    /// Update with a new long description
    #[must_use]
    fn long_description(self, long_desc: impl Into<Cow<'text, str>>) -> Self;

    /// Extend the suggestions with the given suggestions, does not remove any previously added suggestions
    #[must_use]
    fn suggestions(self, suggestions: impl IntoIterator<Item = impl Into<Cow<'text, str>>>)
        -> Self;

    /// Set the version of the underlying format
    #[must_use]
    fn version(self, version: impl Into<Cow<'text, str>>) -> Self;

    /// Update with a new context
    #[must_use]
    fn context(self, context: Context<'text>) -> Self;

    /// Add the given underlying errors, will append to the current list.
    #[must_use]
    fn add_underlying_errors(
        self,
        underlying_errors: impl IntoIterator<Item = CustomError<'text>>,
    ) -> Self;

    /// Add the given underlying error, will append to the current list.
    #[must_use]
    fn add_underlying_error(self, underlying_error: CustomError<'text>) -> Self;

    /// Set the context line index
    #[must_use]
    fn overwrite_line_index(self, line_index: u32) -> Self;

    /// Tests if this errors is a warning
    fn is_warning(&self) -> bool;

    /// Gives the short description or title for this error
    fn get_short_description(&self) -> &str;

    /// Gives the long description for this error
    fn get_long_description(&self) -> &str;

    /// The suggestions
    fn get_suggestions(&self) -> &[Cow<'text, str>];

    /// The version
    fn get_version(&self) -> &str;

    /// Gives the context for this error
    fn get_context(&self) -> &Context<'text>;

    /// Gives the underlying errors
    fn get_underlying_errors(&self) -> &[CustomError<'text>];
}
