use std::borrow::Cow;

use crate::{Context, CustomError};

pub trait ErrorPayload {
    type Settings;
    fn is_error(&self, settings: Self::Settings) -> bool;
    fn ignored(&self, settings: Self::Settings) -> bool;
}

pub trait ErrorPayloadExt: ErrorPayload {
    fn short_description(&self) -> &'static str;
    fn long_description(&self) -> &'static str;
    fn create_error(
        &self,
        settings: Self::Settings,
        context: Context<'static>,
    ) -> CustomError<'static> {
        CustomError {
            warning: self.is_error(settings),
            short_description: Cow::Borrowed(self.short_description()),
            long_description: Cow::Borrowed(self.long_description()),
            suggestions: Vec::new(),
            version: Cow::default(),
            contexts: vec![context],
            underlying_errors: Vec::new(),
        }
    }
}

pub enum NoPayload {
    Error,
    Warning,
}

impl ErrorPayload for NoPayload {
    type Settings = ();
    fn is_error(&self, _settings: Self::Settings) -> bool {
        matches!(self, Self::Error)
    }
    fn ignored(&self, _settings: Self::Settings) -> bool {
        false
    }
}
