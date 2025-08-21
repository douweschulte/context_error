/// The kind of an error
pub trait ErrorKind: PartialEq + Default {
    /// Support for a settings object, which can be used to change the behaviour of this error
    /// based on user settings. If not used just use `()`.
    type Settings: Clone;

    /// Get the term describing this error, for example 'error' or 'warning'
    fn descriptor(&self) -> &'static str;

    /// Check if this is an error, and so should block succeeding the operation
    fn is_error(&self, settings: Self::Settings) -> bool;

    /// Check if this error can be ignored, meaning fully deleted when combining the errors
    fn ignored(&self, settings: Self::Settings) -> bool;
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BasicKind {
    #[default]
    Error,
    Warning,
}

impl ErrorKind for BasicKind {
    type Settings = ();
    fn descriptor(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
    fn is_error(&self, _settings: Self::Settings) -> bool {
        matches!(self, Self::Error)
    }
    fn ignored(&self, _settings: Self::Settings) -> bool {
        false
    }
}

impl std::fmt::Display for BasicKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.descriptor())
    }
}
