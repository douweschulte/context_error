use super::Context;
use serde::*;
use std::borrow::Cow;
use std::error;
use std::fmt;

/// An error. Stored as a pointer to a structure on the heap to prevent large sizes which could be
/// detrimental to performance for the happy path.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CustomError<'text> {
    content: Box<InnerError<'text>>,
}

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct InnerError<'text> {
    /// The level of the error, defining how it should be handled
    warning: bool,
    /// A short description of the error, used as title line
    short_description: Cow<'text, str>,
    /// A longer description of the error, presented below the context to give more information and helpful feedback
    long_description: Cow<'text, str>,
    /// Possible suggestion(s) for the indicated text
    suggestions: Vec<Cow<'text, str>>,
    /// Version if applicable
    version: Cow<'text, str>,
    /// The context, in the most general sense this produces output which leads the user to the right place in the code or file
    context: Context<'text>,
    /// Underlying errors
    underlying_errors: Vec<CustomError<'text>>,
}

impl<'text> CustomError<'text> {
    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `short_desc` - A short description of the error, used as title line.
    /// * `long_desc` -  A longer description of the error, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    pub fn error(
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self {
        Self {
            content: Box::new(InnerError {
                warning: false,
                short_description: short_desc.into(),
                long_description: long_desc.into(),
                suggestions: Vec::new(),
                version: Cow::Borrowed(""),
                context,
                underlying_errors: Vec::new(),
            }),
        }
    }
    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `short_desc` - A short description of the error, generally used as title line.
    /// * `long_desc` -  A longer description of the error, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    pub fn warning(
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self {
        Self {
            content: Box::new(InnerError {
                warning: true,
                short_description: short_desc.into(),
                long_description: long_desc.into(),
                suggestions: Vec::new(),
                version: Cow::Borrowed(""),
                context,
                underlying_errors: Vec::new(),
            }),
        }
    }

    /// The level of the error
    pub const fn level(&self) -> &str {
        if self.content.warning {
            "warning"
        } else {
            "error"
        }
    }

    /// The suggestions
    pub fn suggestions(&self) -> &[Cow<'text, str>] {
        &self.content.suggestions
    }

    /// Tests if this errors is a warning
    pub const fn is_warning(&self) -> bool {
        self.content.warning
    }

    /// Gives the short description or title for this error
    pub fn short_description(&self) -> &str {
        &self.content.short_description
    }

    /// Gives the long description for this error
    pub fn long_description(&self) -> &str {
        &self.content.long_description
    }

    /// Create a copy of the error with a new long description
    #[must_use]
    pub fn with_long_description(&self, long_desc: impl Into<Cow<'text, str>>) -> Self {
        Self {
            content: Box::new(InnerError {
                long_description: long_desc.into(),
                ..(*self.content).clone()
            }),
        }
    }

    /// Create a copy of the error with the given suggestions
    #[must_use]
    pub fn with_suggestions(
        self,
        suggestions: impl IntoIterator<Item = impl Into<Cow<'text, str>>>,
    ) -> Self {
        Self {
            content: Box::new(InnerError {
                suggestions: suggestions.into_iter().map(|s| s.into()).collect(),
                ..(*self.content)
            }),
        }
    }

    /// Set the version of the underlying format
    #[must_use]
    pub fn with_version(self, version: impl Into<Cow<'text, str>>) -> Self {
        Self {
            content: Box::new(InnerError {
                version: version.into(),
                ..(*self.content)
            }),
        }
    }

    /// Create a copy of the error with a new context
    #[must_use]
    pub fn with_context(self, context: Context<'text>) -> Self {
        Self {
            content: Box::new(InnerError {
                context,
                ..(*self.content)
            }),
        }
    }

    /// Create a copy of the error with the given underlying errors
    #[must_use]
    pub fn with_underlying_errors(self, underlying_errors: Vec<Self>) -> Self {
        Self {
            content: Box::new(InnerError {
                underlying_errors,
                ..(*self.content)
            }),
        }
    }

    /// Overwrite the line number with the given number, if applicable
    #[must_use]
    pub fn overwrite_line_number(self, line_number: usize) -> Self {
        Self {
            content: Box::new(InnerError {
                context: self
                    .content
                    .context
                    .clone()
                    .overwrite_line_number(line_number),
                ..(*self.content)
            }),
        }
    }

    /// Gives the context for this error
    pub const fn context(&self) -> &Context<'text> {
        &self.content.context
    }
}

impl fmt::Debug for CustomError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: {}{}\n{}",
            self.level(),
            self.content.short_description,
            self.content.context,
            self.content.long_description
        )?;
        match self.content.suggestions.len() {
            0 => Ok(()),
            1 => writeln!(f, "Did you mean: {}?", self.content.suggestions[0]),
            _ => writeln!(
                f,
                "Did you mean any of: {}?",
                self.content.suggestions.join(", ")
            ),
        }?;
        if !self.content.version.is_empty() {
            writeln!(f, "Version: {}", self.content.version)?;
        }
        match self.content.underlying_errors.len() {
            0 => Ok(()),
            1 => writeln!(
                f,
                "Underlying error:\n{}",
                self.content.underlying_errors[0]
            ),
            _ => writeln!(
                f,
                "Underlying errors:\n{}",
                self.content
                    .underlying_errors
                    .iter()
                    .fold((true, String::new()), |(first, mut acc), el| {
                        if !first {
                            acc.push('\n');
                        }
                        acc.push_str(&el.to_string());
                        (false, acc)
                    })
                    .1
            ),
        }
    }
}

impl fmt::Display for CustomError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for CustomError<'_> {}

#[cfg(test)]
#[expect(clippy::print_stdout)]
mod tests {
    use super::*;
    use crate::FilePosition;

    #[test]
    fn create_empty_error() {
        let a = CustomError::error("test", "test", Context::none());
        println!("{a}");
        assert_eq!(format!("{a}"), "error: test\ntest\n");
        assert!(!a.is_warning());
    }

    #[test]
    fn create_full_line_error() {
        let a = CustomError::warning("test", "test", Context::full_line(0, "testing line"));
        println!("{a}");
        assert_eq!(
            format!("{a}"),
            "warning: test\n  ╷\n1 │ testing line\n  ╵\ntest\n"
        );
        assert!(a.is_warning());
    }

    #[test]
    fn create_range_error() {
        let pos1 = FilePosition {
            text: "hello world\nthis is a multiline\npiece of teXt",
            line_index: 0,
            column: 0,
        };
        let pos2 = FilePosition {
            text: "",
            line_index: 3,
            column: 13,
        };
        let a = CustomError::warning("test", "test error", Context::range(&pos1, &pos2));
        println!("{a}");
        assert_eq!(
            format!("{a}"),
            "warning: test\n  ╷\n1 │ hello world\n2 │ this is a multiline\n3 │ piece of teXt\n  ╵\ntest error\n"
        );
        assert!(a.is_warning());
        assert_eq!(pos2.text, "");
        assert_eq!(pos2.line_index, 3);
        assert_eq!(pos2.column, 13);
    }
}
