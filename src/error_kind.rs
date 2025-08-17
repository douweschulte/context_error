pub trait ErrorKind: PartialEq + Default {
    type Settings: Clone;
    fn is_error(&self, settings: Self::Settings) -> bool;
    fn ignored(&self, settings: Self::Settings) -> bool;
}

pub trait ErrorKindStaticText: ErrorKind {
    fn short_description(&self) -> &'static str;
    fn long_description(&self) -> &'static str;
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
