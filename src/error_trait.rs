use std::borrow::Cow;

use crate::{Coloured, Context, ErrorContent, ErrorKind};

/// A trait to guarantee identical an API between the boxed and unboxed error version
pub trait CustomErrorTrait<'text, Kind>: Sized + Default + PartialEq + ErrorContent<'text>
where
    Kind: ErrorKind,
{
    type UnderlyingError: CustomErrorTrait<'text, Kind> + Clone + PartialEq;

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

    /// Tests if this errors is a warning
    fn get_kind(&self) -> &Kind;

    /// Gives the context for this error
    fn get_contexts(&self) -> &[Context<'text>];

    /// Gives the underlying errors
    fn get_underlying_errors<'a>(&'a self) -> Cow<'a, [Self::UnderlyingError]>;

    /// Check if these two can be merged
    fn could_merge(&self, other: &Self) -> bool {
        self.get_kind() == other.get_kind()
            && self.get_underlying_errors() == other.get_underlying_errors()
            && ErrorContent::could_merge(self, other)
    }

    /// Display this error nicely (used for debug and normal display)
    fn display(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        settings: Option<<Kind as ErrorKind>::Settings>,
    ) -> std::fmt::Result {
        writeln!(
            f,
            "{}: {}",
            if let Some(settings) = settings.clone() {
                if self.get_kind().is_error(settings) {
                    "warning".yellow()
                } else {
                    "error".red()
                }
            } else {
                "error".red()
            },
            self.get_short_description(),
        )?;
        let last = self.get_contexts().len().saturating_sub(1);
        let margin = self
            .get_contexts()
            .iter()
            .map(|c| c.margin())
            .max()
            .unwrap_or_default();
        let mut first = true;
        for (index, context) in self.get_contexts().iter().enumerate() {
            if !context.is_empty() {
                let merged = match (first, index == last) {
                    (true, true) => crate::Merged::No,
                    (true, false) => crate::Merged::First(margin),
                    (false, false) => crate::Merged::Middle(margin),
                    (false, true) => crate::Merged::Last(margin),
                };
                context.display(f, None, merged)?;
                if merged.trailing_decoration() {
                    writeln!(f)?
                };
                first = false;
            }
        }
        writeln!(f, "{}", self.get_long_description())?;
        match self.get_suggestions().len() {
            0 => Ok(()),
            1 => writeln!(
                f,
                "{}: {}?",
                "Did you mean".blue(),
                self.get_suggestions()[0]
            ),
            _ => writeln!(
                f,
                "{}: {}?",
                "Did you mean any of".blue(),
                self.get_suggestions().join(", ")
            ),
        }?;
        if !self.get_version().is_empty() {
            writeln!(f, "{}: {}", "Version".green(), self.get_version())?;
        }
        match self.get_underlying_errors().len() {
            0 => Ok(()),
            1 => {
                writeln!(f, "{}:", "Underlying error".yellow(),)?;
                self.get_underlying_errors()[0].display(f, settings)
            }
            _ => {
                writeln!(f, "{}:", "Underlying errors".yellow(),)?;
                let mut first = true;
                for error in self.get_underlying_errors().iter() {
                    if !first {
                        writeln!(f)?;
                    }
                    error.display(f, settings.clone())?;
                    first = false;
                }
                Ok(())
            }
        }
    }

    fn display_html(
        &self,
        f: &mut impl std::fmt::Write,
        settings: Option<<Kind as ErrorKind>::Settings>,
    ) -> std::fmt::Result {
        write!(
            f,
            "<div class='{}'>",
            if let Some(settings) = settings.clone() {
                if self.get_kind().is_error(settings) {
                    "warning"
                } else {
                    "error"
                }
            } else {
                "error"
            }
        )?;

        write!(f, "<p class='title'>{}</p>", self.get_short_description())?;

        write!(f, "<div class='contexts'>")?;
        for context in self.get_contexts() {
            context.display_html(f)?;
        }
        write!(f, "</div>")?;

        write!(
            f,
            "<p class='description'>{}</p>",
            self.get_long_description()
        )?;
        if !self.get_suggestions().is_empty() {
            write!(
                f,
                "<p>Did you mean{}?</p><ul>",
                if self.get_suggestions().len() == 1 {
                    ""
                } else {
                    " any of"
                }
            )?;
            for suggestion in self.get_suggestions().iter() {
                write!(f, "<li class='suggestion'>{suggestion}</li>")?;
            }
            write!(f, "</ul>")?;
        }
        if !self.get_version().is_empty() {
            write!(
                f,
                "<p class='version'>Version: <span class='version-text'>{}</span></p>",
                self.get_version()
            )?;
        }
        if !self.get_underlying_errors().is_empty() {
            write!(
                f,
                "<label><input type='checkbox'></input> Underlying error{}</label><ul>",
                if self.get_suggestions().len() == 1 {
                    ""
                } else {
                    "s"
                }
            )?;
            for error in self.get_underlying_errors().iter() {
                write!(f, "<li class='underlying_error'>")?;
                error.display_html(f, settings.clone())?;
                write!(f, "</li>")?;
            }
            write!(f, "</ul>")?;
        }

        write!(f, "</div>",)?;
        Ok(())
    }

    fn to_html(&self) -> String {
        let mut string = String::new();
        self.display_html(&mut string, None)
            .expect("Errored while writing to string");
        string
    }
}

impl<'a, T, Kind> CustomErrorTraitExt<'a, Kind> for T
where
    T: CustomErrorTrait<'a, Kind>,
    T::UnderlyingError: CustomErrorTrait<'a, Kind>,
    Kind: ErrorContent<'a> + ErrorKind,
{
}

pub trait CustomErrorTraitExt<'a, Kind>: CustomErrorTrait<'a, Kind>
where
    Kind: ErrorContent<'a> + ErrorKind,
{
    fn from_kind(kind: Kind, context: Context<'a>) -> Self {
        let short_desc = kind.get_short_description();
        let long_desc = kind.get_long_description();
        let suggestions = kind.get_suggestions();
        let version = kind.get_version();
        Self::new(kind, short_desc, long_desc, context)
            .suggestions(suggestions.iter().cloned())
            .version(version)
    }
}
