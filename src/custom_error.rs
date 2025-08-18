use std::{borrow::Cow, error, fmt};

use crate::{BoxedError, Context, CreateError, ErrorKind, FullErrorContent, StaticErrorContent};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CustomError<'text, Kind> {
    /// The kind of the error
    pub(crate) kind: Kind,
    /// A short description of the error, used as title line
    pub(crate) short_description: Cow<'text, str>,
    /// A longer description of the error, presented below the context to give more information and helpful feedback
    pub(crate) long_description: Cow<'text, str>,
    /// Possible suggestion(s) for the indicated text
    pub(crate) suggestions: Vec<Cow<'text, str>>,
    /// Version if applicable
    pub(crate) version: Cow<'text, str>,
    /// The context, in the most general sense this produces output which leads the user to the right place in the code or file
    pub(crate) contexts: Vec<Context<'text>>,
    /// Underlying errors
    pub(crate) underlying_errors: Vec<CustomError<'text, Kind>>,
}

impl<'text, Kind: 'text> StaticErrorContent<'text> for CustomError<'text, Kind> {
    /// Gives the short description or title for this error
    fn get_short_description(&self) -> Cow<'text, str> {
        self.short_description.clone()
    }

    /// Gives the long description for this error
    fn get_long_description(&self) -> Cow<'text, str> {
        self.long_description.clone()
    }

    /// The suggestions
    fn get_suggestions<'a>(&'a self) -> Cow<'a, [Cow<'text, str>]> {
        Cow::Borrowed(self.suggestions.as_slice())
    }

    /// The version
    fn get_version(&self) -> Cow<'text, str> {
        self.version.clone()
    }
}

impl<'text, Kind: 'text + Clone + PartialEq + ErrorKind> FullErrorContent<'text, Kind>
    for CustomError<'text, Kind>
{
    type UnderlyingError = Self;

    fn get_kind(&self) -> Kind {
        self.kind.clone()
    }

    /// Gives the context for this error
    fn get_contexts<'a>(&'a self) -> Cow<'a, [Context<'text>]> {
        Cow::Borrowed(self.contexts.as_slice())
    }

    /// Gives the underlying errors
    fn get_underlying_errors<'a>(&'a self) -> Cow<'a, [Self::UnderlyingError]> {
        Cow::Borrowed(self.underlying_errors.as_slice())
    }
}

impl<'text, Kind: ErrorKind + 'text + Clone> CreateError<'text, Kind> for CustomError<'text, Kind> {
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
        CustomError {
            kind,
            short_description: short_desc.into(),
            long_description: long_desc.into(),
            contexts: vec![context],
            ..Default::default()
        }
    }

    /// Update with a new long description
    fn long_description(self, long_desc: impl Into<Cow<'text, str>>) -> Self {
        Self {
            long_description: long_desc.into(),
            ..self
        }
    }

    /// Extend the suggestions with the given suggestions, does not remove any previously added suggestions
    fn suggestions(
        mut self,
        suggestions: impl IntoIterator<Item = impl Into<Cow<'text, str>>>,
    ) -> Self {
        self.suggestions
            .extend(suggestions.into_iter().map(|s| s.into()));
        self
    }

    /// Set the version of the underlying format
    fn version(self, version: impl Into<Cow<'text, str>>) -> Self {
        Self {
            version: version.into(),
            ..self
        }
    }

    /// Update with a new context
    fn replace_context(self, context: Context<'text>) -> Self {
        Self {
            contexts: vec![context],
            ..self
        }
    }

    /// Add an additional contexts, this should only be used to merge identical errors together.
    fn add_contexts(mut self, contexts: impl IntoIterator<Item = Context<'text>>) -> Self {
        self.contexts.extend(contexts);
        self
    }

    /// Add an additional contexts, this should only be used to merge identical errors together.
    fn add_contexts_ref(&mut self, contexts: impl IntoIterator<Item = Context<'text>>) {
        self.contexts.extend(contexts);
    }

    /// Add an additional context, this should only be used to merge identical errors together.
    fn add_context(mut self, context: Context<'text>) -> Self {
        self.contexts.push(context);
        self
    }

    /// Add the given underlying errors, will append to the current list.
    fn add_underlying_errors(
        mut self,
        underlying_errors: impl IntoIterator<Item = impl Into<CustomError<'text, Kind>>>,
    ) -> Self {
        self.underlying_errors
            .extend(underlying_errors.into_iter().map(|e| e.into()));
        self
    }

    /// Add the given underlying error, will append to the current list.
    fn add_underlying_error(
        mut self,
        underlying_error: impl Into<CustomError<'text, Kind>>,
    ) -> Self {
        self.underlying_errors.push(underlying_error.into());
        self
    }

    /// Set the context line index
    fn overwrite_line_index(self, line_index: u32) -> Self {
        Self {
            contexts: self
                .contexts
                .into_iter()
                .map(|c| c.line_index(line_index))
                .collect(),
            ..self
        }
    }
}

impl<'text, Kind: ErrorKind> CustomError<'text, Kind> {
    /// (Possibly) clone the text to get a static valid error
    pub fn to_owned(self) -> CustomError<'static, Kind> {
        CustomError {
            short_description: Cow::Owned(self.short_description.into_owned()),
            long_description: Cow::Owned(self.long_description.into_owned()),
            suggestions: self
                .suggestions
                .into_iter()
                .map(|p| Cow::Owned(p.into_owned()))
                .collect(),
            version: Cow::Owned(self.version.into_owned()),
            contexts: self.contexts.into_iter().map(|c| c.to_owned()).collect(),
            underlying_errors: self
                .underlying_errors
                .into_iter()
                .map(|e| e.to_owned())
                .collect(),
            ..self
        }
    }
}

impl<Kind: ErrorKind + Clone> fmt::Debug for CustomError<'_, Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, None)
    }
}

impl<Kind: ErrorKind + Clone> fmt::Display for CustomError<'_, Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, None)
    }
}

impl<Kind: ErrorKind + Clone> error::Error for CustomError<'_, Kind> {}

impl<'text, Kind: ErrorKind> From<BoxedError<'text, Kind>> for CustomError<'text, Kind> {
    fn from(value: BoxedError<'text, Kind>) -> Self {
        *value.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasicKind, FilePosition};

    macro_rules! test {
        ($name:ident: $error:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let error = $error;
                let string = error.to_string();
                #[cfg(not(feature="ascii-only"))]
                if string != $expected {
                    panic!("Generated error:\n{}\nNot identical to expected:\n{}\nThis is the generated if this actually is correct: {0:?}", string, $expected);
                }
                crate::context::test_characters(&string);
            }
        };
    }

    test!(empty: CustomError::new(BasicKind::Error, "test", "test", Context::none()) => "error: test\ntest\n");
    test!(full_line: CustomError::new(BasicKind::Warning, "test", "test", Context::full_line(0, "testing line")) 
        => "warning: test\n  ╷\n1 │ testing line\n  ╵\ntest\n");
    test!(range:  CustomError::new(BasicKind::Warning, "test", "test error", Context::range(&FilePosition {text: "hello world\nthis is a multiline\npiece of teXt", line_index: 0, column: 0}, &FilePosition {text: "", line_index: 3, column: 13})) 
        => "warning: test\n  ╷\n1 │ hello world\n2 │ this is a multiline\n3 │ piece of teXt\n  ╵\ntest error\n");
    test!(suggestion: CustomError::new(BasicKind::Error, "Invalid path", "This file does not exist", Context::show("fileee.txt")).suggestions(["file.txt"]) 
        => "error: Invalid path\n ╷\n │ fileee.txt\n ╵\nThis file does not exist\nDid you mean: file.txt?\n");
    test!(suggestions: CustomError::new(BasicKind::Error, "Invalid path", "This file does not exist", Context::show("fileee.txt")).suggestions(["file.txt", "filet.txt"]) 
        => "error: Invalid path\n ╷\n │ fileee.txt\n ╵\nThis file does not exist\nDid you mean any of: file.txt, filet.txt?\n");
    test!(version: CustomError::new(BasicKind::Error, "Invalid number", "This columns is not a number", Context::default().lines(0, "null,80o0,YES,,67.77").add_highlight((0, 5..9))).version("Software AB v2025.42") 
        => "error: Invalid number\n ╷\n │ null,80o0,YES,,67.77\n ╎      ╶──╴\n ╵\nThis columns is not a number\nVersion: Software AB v2025.42\n");
    test!(merged: CustomError::new(BasicKind::Error, "Invalid number", "This columns is not a number", Context::default().line_index(2).lines(0, "null,80o0,YES,,67.77").add_highlight((0, 5..9)))
            .version("Software AB v2025.42")
            .add_context(Context::default().line_index(12).lines(0, "null,7oo1,NO,-1,23.11").add_highlight((0, 5..9)))
            .add_context(Context::default().line_index(34).lines(0, "HOMOSAPIENS,12i1,YES,,1.23").add_highlight((0, 12..16)))
        => "error: Invalid number\n   ╷\n3  │ null,80o0,YES,,67.77\n   ╎      ╶──╴\n13 │ null,7oo1,NO,-1,23.11\n   ╎      ╶──╴\n35 │ HOMOSAPIENS,12i1,YES,,1.23\n   ╎             ╶──╴\n   ╵\nThis columns is not a number\nVersion: Software AB v2025.42\n");

    const TEXT: &str = "number";

    test!(underlying_error: CustomError::new(BasicKind::Error, "Invalid csv line", format!("This column is not a {TEXT}"), Context::default().lines(0, "null,80o0,YES,,67.77").add_highlight((0, 5..9)))
                .add_underlying_error(CustomError::new(BasicKind::Error, "Invalid number", "The number contains invalid digit(s)", Context::default().lines(0, "null,80o0,YES,,67.77").add_highlight((0, 7..8)))) 
            => "error: Invalid csv line\n ╷\n │ null,80o0,YES,,67.77\n ╎      ╶──╴\n ╵\nThis column is not a number\nUnderlying error:\nerror: Invalid number\n ╷\n │ null,80o0,YES,,67.77\n ╎        ⁃\n ╵\nThe number contains invalid digit(s)\n\n");

    #[test]
    fn test_level() {
        let a = CustomError::new(BasicKind::Error, "test", "test", Context::none());
        assert!(a.get_kind().is_error(()));
        let a = CustomError::new(BasicKind::Warning, "test", "test", Context::none());
        assert!(!a.get_kind().is_error(()));
    }

    #[test]
    fn test_well_behaved() {
        let a = CustomError::new(BasicKind::Error, "test", "test", Context::none());
        let _io_packaged = std::io::Error::other(a);
    }
}
