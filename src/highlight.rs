use std::{
    borrow::Cow,
    ops::{Bound, RangeBounds},
};

/// A highlight on a single line. The easiest way of creating these is by using the [From] implementations.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Highlight<'text> {
    /// Line index in case multiple lines are given
    pub line: u8,
    /// The offset (in chars) into the line
    pub offset: u32,
    /// The length of the highlight
    pub length: u8,
    /// Optional comment to post next to the highlight
    pub comment: Option<Cow<'text, str>>,
}

/// Create a highlight at the given line, offset, and of the given length without a comment.
impl From<(u8, u32, u8)> for Highlight<'static> {
    fn from(value: (u8, u32, u8)) -> Self {
        Self {
            line: value.0,
            offset: value.1,
            length: value.2,
            comment: None,
        }
    }
}

/// Create a highlight at the given line, offset, of the given length, and with a comment.
impl<'text, Comment: Into<Cow<'text, str>>> From<(u8, u32, u8, Comment)> for Highlight<'text> {
    fn from(value: (u8, u32, u8, Comment)) -> Self {
        Self {
            line: value.0,
            offset: value.1,
            length: value.2,
            comment: Some(value.3.into()),
        }
    }
}

/// Create a highlight at the given line and at the given range, without a comment.
impl<'text, Range: RangeBounds<u32>> From<(u8, Range)> for Highlight<'text> {
    fn from(value: (u8, Range)) -> Self {
        let offset = match value.1.start_bound() {
            Bound::Excluded(n) => n + 1,
            Bound::Included(n) => *n,
            Bound::Unbounded => 0,
        };
        Self {
            line: value.0,
            offset,
            length: match value.1.end_bound() {
                Bound::Excluded(n) => u8::try_from(n - offset).unwrap_or(u8::MAX),
                Bound::Included(n) => u8::try_from(n - offset + 1).unwrap_or(u8::MAX),
                Bound::Unbounded => u8::MAX,
            },
            comment: None,
        }
    }
}

/// Create a highlight at the given line, at the given range, and with a comment.
/// Used `u32` here because otherwise this clashes with the `(usize, usize, usize)` option.
impl<'text, Range: RangeBounds<u32>, Comment: Into<Cow<'text, str>>> From<(u16, Range, Comment)>
    for Highlight<'text>
{
    fn from(value: (u16, Range, Comment)) -> Self {
        let offset = match value.1.start_bound() {
            Bound::Excluded(n) => n + 1,
            Bound::Included(n) => *n,
            Bound::Unbounded => 0,
        };
        Self {
            line: value.0 as u8,
            offset,
            length: match value.1.end_bound() {
                Bound::Excluded(n) => u8::try_from(n - offset).unwrap_or(u8::MAX),
                Bound::Included(n) => u8::try_from(n - offset + 1).unwrap_or(u8::MAX),
                Bound::Unbounded => u8::MAX,
            },
            comment: Some(value.2.into()),
        }
    }
}

impl<'text> Highlight<'text> {
    /// (Possibly) clone the comment to get a static valid highlight
    pub fn to_owned(self) -> Highlight<'static> {
        Highlight {
            comment: self.comment.map(|c| Cow::Owned(c.into_owned())),
            ..self
        }
    }
}
