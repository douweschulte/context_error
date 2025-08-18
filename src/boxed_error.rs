use core::fmt;
use std::{borrow::Cow, error};

use crate::{Context, CreateError, CustomError, ErrorKind, FullErrorContent, StaticErrorContent};

/// An error. Stored as a pointer to a structure on the heap to prevent large sizes which could be
/// detrimental to performance for the happy path.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoxedError<'text, Kind> {
    pub(crate) content: Box<CustomError<'text, Kind>>,
}

impl<'text, Kind: 'text> StaticErrorContent<'text> for BoxedError<'text, Kind> {
    /// Gives the short description or title for this error
    fn get_short_description(&self) -> Cow<'text, str> {
        self.content.short_description.clone()
    }

    /// Gives the long description for this error
    fn get_long_description(&self) -> Cow<'text, str> {
        self.content.long_description.clone()
    }

    /// The suggestions
    fn get_suggestions<'a>(&'a self) -> Cow<'a, [Cow<'text, str>]> {
        Cow::Borrowed(self.content.suggestions.as_slice())
    }

    /// The version
    fn get_version(&self) -> Cow<'text, str> {
        self.content.version.clone()
    }
}

impl<'text, Kind: 'text + Clone + PartialEq + ErrorKind> FullErrorContent<'text, Kind>
    for BoxedError<'text, Kind>
{
    type UnderlyingError = CustomError<'text, Kind>;

    fn get_kind(&self) -> Kind {
        self.content.kind.clone()
    }

    /// Gives the context for this error
    fn get_contexts<'a>(&'a self) -> Cow<'a, [Context<'text>]> {
        Cow::Borrowed(self.content.contexts.as_slice())
    }

    /// Gives the underlying errors
    fn get_underlying_errors<'a>(&'a self) -> Cow<'a, [Self::UnderlyingError]> {
        Cow::Borrowed(self.content.underlying_errors.as_slice())
    }
}

impl<'text, Kind: ErrorKind + 'text + Clone> CreateError<'text, Kind> for BoxedError<'text, Kind> {
    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `short_desc` - A short description of the error, used as title line.
    /// * `long_desc` - A longer description of the error, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    fn new(
        kind: Kind,
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self {
        Self {
            content: Box::new(CustomError::new(kind, short_desc, long_desc, context)),
        }
    }

    /// Update with a new long description
    fn long_description(mut self, long_desc: impl Into<Cow<'text, str>>) -> Self {
        self.content.long_description = long_desc.into();
        self
    }

    /// Extend the suggestions with the given suggestions, does not remove any previously added suggestions
    fn suggestions(
        mut self,
        suggestions: impl IntoIterator<Item = impl Into<Cow<'text, str>>>,
    ) -> Self {
        self.content
            .suggestions
            .extend(suggestions.into_iter().map(|s| s.into()));
        self
    }

    /// Set the version of the underlying format
    fn version(mut self, version: impl Into<Cow<'text, str>>) -> Self {
        self.content.version = version.into();
        self
    }

    /// Update with a new context
    fn replace_context(mut self, context: Context<'text>) -> Self {
        self.content.contexts = vec![context];
        self
    }

    /// Add an additional contexts, this should only be used to merge identical errors together.
    fn add_contexts(mut self, contexts: impl IntoIterator<Item = Context<'text>>) -> Self {
        self.content.contexts.extend(contexts);
        self
    }

    /// Add an additional contexts, this should only be used to merge identical errors together.
    fn add_contexts_ref(&mut self, contexts: impl IntoIterator<Item = Context<'text>>) {
        self.content.contexts.extend(contexts);
    }

    /// Add an additional context, this should only be used to merge identical errors together.
    fn add_context(mut self, context: Context<'text>) -> Self {
        self.content.contexts.push(context);
        self
    }

    /// Add the given underlying errors, will append to the current list.
    fn add_underlying_errors(
        mut self,
        underlying_errors: impl IntoIterator<Item = impl Into<Self::UnderlyingError>>,
    ) -> Self {
        self.content
            .underlying_errors
            .extend(underlying_errors.into_iter().map(|e| e.into()));
        self
    }

    /// Add the given underlying error, will append to the current list.
    fn add_underlying_error(mut self, underlying_error: impl Into<Self::UnderlyingError>) -> Self {
        self.content.underlying_errors.push(underlying_error.into());
        self
    }

    /// Set the context line index
    fn overwrite_line_index(mut self, line_index: u32) -> Self {
        self.content.contexts = self
            .content
            .contexts
            .into_iter()
            .map(|c| c.line_index(line_index))
            .collect();
        self
    }
}

impl<'text, Kind: ErrorKind> BoxedError<'text, Kind> {
    /// (Possibly) clone the text to get a static valid error
    pub fn to_owned(self) -> BoxedError<'static, Kind> {
        BoxedError {
            content: Box::new((*self.content).to_owned()),
        }
    }
}

impl<Kind: ErrorKind + Clone> fmt::Debug for BoxedError<'_, Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, None)
    }
}

impl<Kind: ErrorKind + Clone> fmt::Display for BoxedError<'_, Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, None)
    }
}

impl<Kind: ErrorKind + Clone> error::Error for BoxedError<'_, Kind> {}

impl<'text, Kind: ErrorKind> From<CustomError<'text, Kind>> for BoxedError<'text, Kind> {
    fn from(value: CustomError<'text, Kind>) -> Self {
        Self {
            content: Box::new(value),
        }
    }
}
