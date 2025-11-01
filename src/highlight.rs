use std::{
    borrow::Cow,
    ops::{Bound, RangeBounds},
};

/// A highlight on a single line. The easiest way of creating these is by using the [From] implementations.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Highlight<'text> {
    /// Line index in case multiple lines are given
    pub line: usize,
    /// The offset (in chars) into the line
    pub offset: usize,
    /// The length of the highlight
    pub length: usize,
    /// Optional comment to post next to the highlight
    pub comment: Option<Cow<'text, str>>,
}

/// Create a highlight at the given line, offset, and of the given length without a comment.
impl<'text> From<(usize, usize, usize)> for Highlight<'text> {
    fn from(value: (usize, usize, usize)) -> Self {
        Self {
            line: value.0,
            offset: value.1,
            length: value.2,
            comment: None,
        }
    }
}

/// Create a highlight at the given line, offset, of the given length, and with a comment.
impl<'text, Comment: Into<Cow<'text, str>>> From<(usize, usize, usize, Comment)>
    for Highlight<'text>
{
    fn from(value: (usize, usize, usize, Comment)) -> Self {
        Self {
            line: value.0,
            offset: value.1,
            length: value.2,
            comment: Some(value.3.into()),
        }
    }
}

/// Create a highlight at the given line and at the given range, without a comment.
impl<'text, Range: RangeBounds<usize>> From<(usize, Range)> for Highlight<'text> {
    fn from(value: (usize, Range)) -> Self {
        let offset = match value.1.start_bound() {
            Bound::Excluded(n) => n + 1,
            Bound::Included(n) => *n,
            Bound::Unbounded => 0,
        };
        Self {
            line: value.0,
            offset,
            length: match value.1.end_bound() {
                Bound::Excluded(n) => n.saturating_sub(offset),
                Bound::Included(n) => n.saturating_sub(offset + 1),
                Bound::Unbounded => usize::MAX,
            },
            comment: None,
        }
    }
}

/// Create a highlight at the given line, at the given range, and with a comment.
/// Used `u64` here because otherwise this clashes with the `(usize, usize, usize)` option.
impl<'text, Range: RangeBounds<usize>, Comment: Into<Cow<'text, str>>> From<(u64, Range, Comment)>
    for Highlight<'text>
{
    fn from(value: (u64, Range, Comment)) -> Self {
        let offset = match value.1.start_bound() {
            Bound::Excluded(n) => n + 1,
            Bound::Included(n) => *n,
            Bound::Unbounded => 0,
        };
        Self {
            line: value.0 as usize,
            offset,
            length: match value.1.end_bound() {
                Bound::Excluded(n) => n - offset,
                Bound::Included(n) => n - offset + 1,
                Bound::Unbounded => usize::MAX,
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
