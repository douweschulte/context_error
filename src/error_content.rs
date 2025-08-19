use std::borrow::Cow;

use crate::{Coloured, Context, ErrorKind};

/// A structure that contains basic error content
pub trait StaticErrorContent<'text>
where
    Self: 'text,
{
    /// Gives the short description or title for this error
    fn get_short_description(&self) -> Cow<'text, str>;

    /// Gives the long description for this error
    fn get_long_description(&self) -> Cow<'text, str>;

    /// The suggestions
    fn get_suggestions<'a>(&'a self) -> Cow<'a, [Cow<'text, str>]>;

    /// The version
    fn get_version(&self) -> Cow<'text, str>;

    /// Check if these two can be merged
    fn could_merge(&self, other: &Self) -> bool {
        self.get_short_description() == other.get_short_description()
            && self.get_long_description() == other.get_long_description()
            && self.get_suggestions() == other.get_suggestions()
            && self.get_version() == other.get_version()
    }

    /// Display this error nicely (used for debug and normal display)
    fn display_with_context<Kind: ErrorKind, UnderlyingError: FullErrorContent<'text, Kind>>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        kind: Kind,
        settings: Option<<Kind as ErrorKind>::Settings>,
        contexts: &[Context<'text>],
        underlying_errors: &[UnderlyingError],
    ) -> std::fmt::Result {
        writeln!(
            f,
            "{}: {}",
            if settings
                .clone()
                .map_or(true, |settings| kind.is_error(settings))
            {
                kind.descriptor().red()
            } else {
                kind.descriptor().blue()
            },
            self.get_short_description(),
        )?;
        let last = contexts.len().saturating_sub(1);
        let margin = contexts
            .iter()
            .map(|c| c.margin())
            .max()
            .unwrap_or_default();
        let mut first = true;
        for (index, context) in contexts.iter().enumerate() {
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
        match underlying_errors.len() {
            0 => Ok(()),
            1 => {
                writeln!(f, "{}:", "Underlying error".yellow(),)?;
                underlying_errors[0].display(f, settings)
            }
            _ => {
                writeln!(f, "{}:", "Underlying errors".yellow(),)?;
                let mut first = true;
                for error in underlying_errors.iter() {
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

    fn display_html_with_context<
        Kind: ErrorKind,
        UnderlyingError: FullErrorContent<'text, Kind>,
    >(
        &self,
        f: &mut impl std::fmt::Write,
        kind: Kind,
        settings: Option<<Kind as ErrorKind>::Settings>,
        contexts: &[Context<'text>],
        underlying_errors: &[UnderlyingError],
    ) -> std::fmt::Result {
        write!(f, "<div class='{}'>", kind.descriptor(),)?;

        write!(f, "<p class='title'>{}</p>", self.get_short_description())?;

        write!(f, "<div class='contexts'>")?;
        for context in contexts.iter() {
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
        if !underlying_errors.is_empty() {
            write!(
                f,
                "<label><input type='checkbox'></input> Underlying error{}</label><ul>",
                if self.get_suggestions().len() == 1 {
                    ""
                } else {
                    "s"
                }
            )?;
            for error in underlying_errors.iter() {
                write!(f, "<li class='underlying_error'>")?;
                error.display_html(f, settings.clone())?;
                write!(f, "</li>")?;
            }
            write!(f, "</ul>")?;
        }

        write!(f, "</div>",)?;
        Ok(())
    }
}

/// A structure that contains all error content
pub trait FullErrorContent<'text, Kind>: StaticErrorContent<'text>
where
    Kind: ErrorKind,
{
    type UnderlyingError: FullErrorContent<'text, Kind> + Clone + PartialEq;

    fn get_kind(&self) -> Kind;

    /// Get the context of the error
    fn get_contexts<'a>(&'a self) -> Cow<'a, [Context<'text>]>;

    /// The underlying errors
    fn get_underlying_errors<'a>(&'a self) -> Cow<'a, [Self::UnderlyingError]>;

    /// Check if these two can be merged
    fn could_merge(&self, other: &Self) -> bool {
        self.get_kind() == other.get_kind()
            && self.get_underlying_errors() == other.get_underlying_errors()
            && StaticErrorContent::could_merge(self, other)
    }

    /// Display this error nicely in text
    fn display(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        settings: Option<<Kind as ErrorKind>::Settings>,
    ) -> std::fmt::Result {
        self.display_with_context(
            f,
            self.get_kind(),
            settings,
            &self.get_contexts(),
            &self.get_underlying_errors(),
        )
    }

    /// Display this error nicely in HTML
    fn display_html(
        &self,
        f: &mut impl std::fmt::Write,
        settings: Option<<Kind as ErrorKind>::Settings>,
    ) -> std::fmt::Result {
        self.display_html_with_context(
            f,
            self.get_kind(),
            settings,
            &self.get_contexts(),
            &self.get_underlying_errors(),
        )
    }

    /// Display this error nicely in HTML as a convenience method (similar to `to_string` which is automatically made if you support `Display`)
    fn to_html(&self) -> String {
        let mut string = String::new();
        self.display_html(&mut string, None)
            .expect("Errored while writing to string");
        string
    }
}
