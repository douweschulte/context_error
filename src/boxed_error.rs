use core::fmt;
use std::{borrow::Cow, error};

use crate::{Context, CustomError, CustomErrorTrait};

/// An error. Stored as a pointer to a structure on the heap to prevent large sizes which could be
/// detrimental to performance for the happy path.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoxedError<'text> {
    pub(crate) content: Box<CustomError<'text>>,
}

impl<'text> CustomErrorTrait<'text> for BoxedError<'text> {
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
    ) -> Self {
        Self {
            content: Box::new(CustomError::error(short_desc, long_desc, context)),
        }
    }

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
    ) -> Self {
        Self {
            content: Box::new(CustomError::warning(short_desc, long_desc, context)),
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
        underlying_errors: impl IntoIterator<Item = impl Into<CustomError<'text>>>,
    ) -> Self {
        self.content
            .underlying_errors
            .extend(underlying_errors.into_iter().map(|e| e.into()));
        self
    }

    /// Add the given underlying error, will append to the current list.
    fn add_underlying_error(mut self, underlying_error: impl Into<CustomError<'text>>) -> Self {
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

    /// Tests if this errors is a warning
    fn is_warning(&self) -> bool {
        self.content.warning
    }

    /// Gives the short description or title for this error
    fn get_short_description(&self) -> &str {
        &self.content.short_description
    }

    /// Gives the long description for this error
    fn get_long_description(&self) -> &str {
        &self.content.long_description
    }

    /// The suggestions
    fn get_suggestions(&self) -> &[Cow<'text, str>] {
        &self.content.suggestions
    }

    /// The version
    fn get_version(&self) -> &str {
        &self.content.version
    }

    /// Gives the context for this error
    fn get_contexts(&self) -> &[Context<'text>] {
        &self.content.contexts
    }

    /// Gives the underlying errors
    fn get_underlying_errors(&self) -> &[CustomError<'text>] {
        &self.content.underlying_errors
    }
}

impl<'text> BoxedError<'text> {
    /// (Possibly) clone the text to get a static valid error
    pub fn to_owned(self) -> BoxedError<'static> {
        BoxedError {
            content: Box::new((*self.content).to_owned()),
        }
    }

    /// Display this error nicely (used for debug and normal display)
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content.display(f)
    }
}

impl fmt::Debug for BoxedError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f)
    }
}

impl fmt::Display for BoxedError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f)
    }
}

impl error::Error for BoxedError<'_> {}
