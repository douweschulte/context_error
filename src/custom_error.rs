use std::{borrow::Cow, error, fmt};

use crate::Context;

/// An error. Stored as a pointer to a structure on the heap to prevent large sizes which could be
/// detrimental to performance for the happy path.
pub type BoxedError<'text> = Box<CustomError<'text>>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CustomError<'text> {
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
    /// * `long_desc` - A longer description of the error, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    pub fn error(
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self {
        CustomError {
            warning: false,
            short_description: short_desc.into(),
            long_description: long_desc.into(),
            context,
            ..Default::default()
        }
    }

    /// Create a new `CustomError`.
    ///
    /// ## Arguments
    /// * `short_desc` - A short description of the warning, generally used as title line.
    /// * `long_desc` - A longer description of the warning, presented below the context to give more information and helpful feedback.
    /// * `context` - The context, in the most general sense this produces output which leads the user to the right place in the code or file.
    pub fn warning(
        short_desc: impl Into<Cow<'text, str>>,
        long_desc: impl Into<Cow<'text, str>>,
        context: Context<'text>,
    ) -> Self {
        Self {
            warning: true,
            short_description: short_desc.into(),
            long_description: long_desc.into(),
            context,
            ..Default::default()
        }
    }

    /// Update with a new long description
    #[must_use]
    pub fn long_description(self, long_desc: impl Into<Cow<'text, str>>) -> Self {
        Self {
            long_description: long_desc.into(),
            ..self
        }
    }

    /// Extend the suggestions with the given suggestions, does not remove any previously added suggestions
    #[must_use]
    pub fn suggestions(
        mut self,
        suggestions: impl IntoIterator<Item = impl Into<Cow<'text, str>>>,
    ) -> Self {
        self.suggestions
            .extend(suggestions.into_iter().map(|s| s.into()));
        self
    }

    /// Set the version of the underlying format
    #[must_use]
    pub fn version(self, version: impl Into<Cow<'text, str>>) -> Self {
        Self {
            version: version.into(),
            ..self
        }
    }

    /// Update with a new context
    #[must_use]
    pub fn context(self, context: Context<'text>) -> Self {
        Self { context, ..self }
    }

    /// Add the given underlying errors, will append to the current list.
    #[must_use]
    pub fn add_underlying_errors(
        mut self,
        underlying_errors: impl IntoIterator<Item = Self>,
    ) -> Self {
        self.underlying_errors.extend(underlying_errors);
        self
    }

    /// Add the given underlying error, will append to the current list.
    #[must_use]
    pub fn add_underlying_error(mut self, underlying_error: Self) -> Self {
        self.underlying_errors.push(underlying_error);
        self
    }

    /// Set the context line index
    #[must_use]
    pub fn overwrite_line_index(self, line_index: u32) -> Self {
        Self {
            context: self.context.line_index(line_index),
            ..self
        }
    }

    /// Tests if this errors is a warning
    pub const fn is_warning(&self) -> bool {
        self.warning
    }

    /// Gives the short description or title for this error
    pub fn get_short_description(&self) -> &str {
        &self.short_description
    }

    /// Gives the long description for this error
    pub fn get_long_description(&self) -> &str {
        &self.long_description
    }

    /// The suggestions
    pub fn get_suggestions(&self) -> &[Cow<'text, str>] {
        &self.suggestions
    }

    /// The version
    pub fn get_version(&self) -> &str {
        &self.version
    }

    /// Gives the context for this error
    pub const fn get_context(&self) -> &Context<'text> {
        &self.context
    }

    /// Gives the underlying errors
    pub fn get_underlying_errors(&self) -> &[Self] {
        &self.underlying_errors
    }

    /// (Possibly) clone the text to get a static valid error
    pub fn to_owned(self) -> CustomError<'static> {
        CustomError {
            short_description: Cow::Owned(self.short_description.into_owned()),
            long_description: Cow::Owned(self.long_description.into_owned()),
            suggestions: self
                .suggestions
                .into_iter()
                .map(|p| Cow::Owned(p.into_owned()))
                .collect(),
            version: Cow::Owned(self.version.into_owned()),
            context: self.context.to_owned(),
            underlying_errors: self
                .underlying_errors
                .into_iter()
                .map(|e| e.to_owned())
                .collect(),
            ..self
        }
    }

    /// Display this error nicely (used for debug and normal display)
    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: {}{}{}\n{}",
            if self.warning { "warning" } else { "error" },
            self.short_description,
            if self.context.is_empty() { "" } else { "\n" },
            self.context,
            self.long_description
        )?;
        match self.suggestions.len() {
            0 => Ok(()),
            1 => writeln!(f, "Did you mean: {}?", self.suggestions[0]),
            _ => writeln!(f, "Did you mean any of: {}?", self.suggestions.join(", ")),
        }?;
        if !self.version.is_empty() {
            writeln!(f, "Version: {}", self.version)?;
        }
        match self.underlying_errors.len() {
            0 => Ok(()),
            1 => writeln!(f, "Underlying error:\n{}", self.underlying_errors[0]),
            _ => writeln!(
                f,
                "Underlying errors:\n{}",
                self.underlying_errors
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

impl fmt::Debug for CustomError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f)
    }
}

impl fmt::Display for CustomError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f)
    }
}

impl error::Error for CustomError<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FilePosition;

    macro_rules! test {
        ($name:ident: $error:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let error = $error;
                let string = error.to_string();
                if string != $expected {
                    panic!("Generated error:\n{}\nNot identical to expected:\n{}\nThis is the generated if this actually is correct: {0:?}", string, $expected);
                }
            }
        };
    }

    test!(empty: CustomError::error("test", "test", Context::none()) => "error: test\ntest\n");
    test!(full_line: CustomError::warning("test", "test", Context::full_line(0, "testing line")) 
        => "warning: test\n  ╷\n1 │ testing line\n  ╵\ntest\n");
    test!(range:  CustomError::warning("test", "test error", Context::range(&FilePosition {text: "hello world\nthis is a multiline\npiece of teXt", line_index: 0, column: 0}, &FilePosition {text: "", line_index: 3, column: 13})) 
        => "warning: test\n  ╷\n1 │ hello world\n2 │ this is a multiline\n3 │ piece of teXt\n  ╵\ntest error\n");
    test!(suggestion: CustomError::error("Invalid path", "This file does not exist", Context::show("fileee.txt")).suggestions(["file.txt"]) 
        => "error: Invalid path\n ╷\n │ fileee.txt\n ╵\nThis file does not exist\nDid you mean: file.txt?\n");
    test!(suggestions: CustomError::error("Invalid path", "This file does not exist", Context::show("fileee.txt")).suggestions(["file.txt", "filet.txt"]) 
        => "error: Invalid path\n ╷\n │ fileee.txt\n ╵\nThis file does not exist\nDid you mean any of: file.txt, filet.txt?\n");
    test!(version: CustomError::error("Invalid number", "This columns is not a number", Context::default().lines(0, "null,80o0,YES,,67.77").add_highlight((0, 5..9))).version("Software AB v2025.42") 
        => "error: Invalid number\n ╷\n │ null,80o0,YES,,67.77\n ╎      ╶──╴\n ╵\nThis columns is not a number\nVersion: Software AB v2025.42\n");

    const TEXT: &str = "number";

    test!(underlying_error: CustomError::error("Invalid csv line", format!("This column is not a {TEXT}"), Context::default().lines(0, "null,80o0,YES,,67.77").add_highlight((0, 5..9)))
                .add_underlying_error(CustomError::error("Invalid number", "The number contains invalid digit(s)", Context::default().lines(0, "null,80o0,YES,,67.77").add_highlight((0, 7..8)))) 
            => "error: Invalid csv line\n ╷\n │ null,80o0,YES,,67.77\n ╎      ╶──╴\n ╵\nThis column is not a number\nUnderlying error:\nerror: Invalid number\n ╷\n │ null,80o0,YES,,67.77\n ╎        ⁃\n ╵\nThe number contains invalid digit(s)\n\n");

    #[test]
    fn test_level() {
        let a = CustomError::error("test", "test", Context::none());
        assert!(!a.is_warning());
        let a = CustomError::warning("test", "test", Context::none());
        assert!(a.is_warning());
    }

    #[test]
    fn test_well_behaved() {
        let a = CustomError::error("test", "test", Context::none());
        let _io_packaged = std::io::Error::other(a);
    }
}
