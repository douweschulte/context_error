use std::borrow::Cow;

use crate::{Context, ErrorKind, FullErrorContent, StaticErrorContent};

/// A trait to guarantee identical an API between the boxed and unboxed error version
pub trait CreateError<'text, Kind>:
    Sized + Default + PartialEq + FullErrorContent<'text, Kind>
where
    Kind: ErrorKind,
{
    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `kind` - The error kind.
    /// * `short_desc` - A short description of the error, used as title line.
    /// * `long_desc` - A longer description of the error, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    fn new(
        kind: Kind,
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self {
        Self::small(kind, short_desc, long_desc).add_context(context)
    }

    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `kind` - The error kind.
    /// * `short_desc` - A short description of the error, used as title line.
    /// * `long_desc` - A longer description of the error, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    fn small(
        kind: Kind,
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
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
    fn replace_context(self, context: Context<'text>) -> Self;

    /// Add an additional contexts, this should only be used to merge identical errors together.
    #[must_use]
    fn add_contexts(self, contexts: impl IntoIterator<Item = Context<'text>>) -> Self;

    /// Add an additional contexts, this should only be used to merge identical errors together.
    fn add_contexts_ref(&mut self, contexts: impl IntoIterator<Item = Context<'text>>);

    /// Add an additional context, this should only be used to merge identical errors together.
    #[must_use]
    fn add_context(self, context: Context<'text>) -> Self;

    /// Add the given underlying errors, will append to the current list.
    #[must_use]
    fn add_underlying_errors(
        self,
        underlying_errors: impl IntoIterator<Item = impl Into<Self::UnderlyingError>>,
    ) -> Self;

    /// Add the given underlying error, will append to the current list.
    #[must_use]
    fn add_underlying_error(self, underlying_error: impl Into<Self::UnderlyingError>) -> Self;

    /// Set the context line index, for every context in this error
    #[must_use]
    fn overwrite_line_index(self, line_index: u32) -> Self;

    /// Create a new error from the given kind
    #[must_use]
    fn from_kind(kind: Kind) -> Self
    where
        Kind: StaticErrorContent<'text>,
    {
        let short_desc = kind.get_short_description();
        let long_desc = kind.get_long_description();
        let suggestions = kind.get_suggestions().to_vec();
        let version = kind.get_version();
        Self::small(kind, short_desc, long_desc)
            .suggestions(suggestions)
            .version(version)
    }

    /// Create a new error from the given kind
    #[must_use]
    fn from_full_kind(kind: Kind) -> Self
    where
        Kind: FullErrorContent<'text, Kind>,
        Kind::UnderlyingError: Into<Self::UnderlyingError>,
    {
        let short_desc = kind.get_short_description();
        let long_desc = kind.get_long_description();
        let suggestions = kind.get_suggestions().to_vec();
        let version = kind.get_version();
        let contexts = kind.get_contexts().to_vec();
        let underlying_errors = kind.get_underlying_errors().to_vec();
        Self::small(kind, short_desc, long_desc)
            .suggestions(suggestions)
            .version(version)
            .add_contexts(contexts)
            .add_underlying_errors(underlying_errors)
    }
}
