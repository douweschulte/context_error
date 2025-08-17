use std::borrow::Cow;

pub trait ErrorKind: PartialEq + Default {
    type Settings: Clone;
    fn is_error(&self, settings: Self::Settings) -> bool;
    fn ignored(&self, settings: Self::Settings) -> bool;
}

pub trait ErrorContent<'text>
where
    Self: 'text,
{
    /// Gives the short description or title for this error
    fn get_short_description(&self) -> Cow<'text, str>;

    /// Gives the long description for this error
    fn get_long_description(&self) -> Cow<'text, str>;

    /// The suggestions
    fn get_suggestions(&self) -> Cow<'text, [Cow<'text, str>]>;

    /// The version
    fn get_version(&self) -> Cow<'text, str>;

    /// Check if these two can be merged
    fn could_merge(&self, other: &Self) -> bool {
        self.get_short_description() == other.get_short_description()
            && self.get_long_description() == other.get_long_description()
            && self.get_suggestions() == other.get_suggestions()
            && self.get_version() == other.get_version()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BasicKind {
    #[default]
    Error,
    Warning,
}

impl ErrorKind for BasicKind {
    type Settings = ();
    fn is_error(&self, _settings: Self::Settings) -> bool {
        matches!(self, Self::Error)
    }
    fn ignored(&self, _settings: Self::Settings) -> bool {
        false
    }
}
