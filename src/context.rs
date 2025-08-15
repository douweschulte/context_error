use std::{
    borrow::Cow,
    fmt,
    num::NonZeroU32,
    ops::{Bound, RangeBounds},
};

use crate::{Coloured, Highlight};

/// A context construct to indicate a context presumably in a file, but could be in any kind of source text
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Context<'text> {
    /// The source or path of the text
    pub(crate) source: Option<Cow<'text, str>>,
    /// 1 based index of the first line (0 is used as niche for the None case)
    pub(crate) line_number: Option<NonZeroU32>,
    /// Offset of the first line (in characters) before the slice starts
    pub(crate) first_line_offset: u32,
    /// The text of this context, multiline text is handled by [str::lines]
    pub(crate) lines: Cow<'text, str>,
    /// The highlights, required to be sorted by line first, offset second
    pub(crate) highlights: Vec<Highlight<'text>>,
}

/// Convenience wrappers using common patterns
impl<'text> Context<'text> {
    /// Creates a new context when no context can be given (identical to [Self::default])
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates a new context when only a line (eg filename) can be shown
    pub fn show(line: impl Into<Cow<'text, str>>) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_number: None,
            lines: line.into(),
            highlights: Vec::new(),
        }
    }

    /// Creates a new context when a full line is faulty and no special position can be annotated
    pub fn full_line(line_index: u32, line: impl Into<Cow<'text, str>>) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_number: NonZeroU32::new(line_index + 1),
            lines: line.into(),
            highlights: Vec::new(),
        }
    }

    /// Creates a new context when a special position can be annotated on a line
    pub fn line(
        line_index: Option<u32>,
        line: impl Into<Cow<'text, str>>,
        offset: usize,
        length: usize,
    ) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_number: line_index.and_then(|i| NonZeroU32::new(i + 1)),
            lines: line.into(),
            highlights: vec![Highlight {
                line: 0,
                offset,
                length,
                comment: None,
            }],
        }
    }

    /// Creates a new context when a special position can be annotated on a line
    pub fn line_with_comment(
        line_index: Option<u32>,
        line: impl Into<Cow<'text, str>>,
        offset: usize,
        length: usize,
        comment: Option<Cow<'text, str>>,
    ) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_number: line_index.and_then(|i| NonZeroU32::new(i + 1)),
            lines: line.into(),
            highlights: vec![Highlight {
                line: 0,
                offset,
                length,
                comment,
            }],
        }
    }

    /// Create a context highlighting a certain range on a single line
    pub fn line_range(
        line_index: Option<u32>,
        line: &'text str,
        range: impl RangeBounds<usize>,
    ) -> Self {
        Self::line_range_with_comment(line_index, line, range, None)
    }

    /// Create a context highlighting a certain range on a single line
    pub fn line_range_with_comment(
        line_index: Option<u32>,
        line: &'text str,
        range: impl RangeBounds<usize>,
        comment: Option<Cow<'text, str>>,
    ) -> Self {
        match (range.start_bound(), range.end_bound()) {
            (Bound::Unbounded, Bound::Unbounded) => {
                line_index.map_or_else(|| Self::show(line), |i| Self::full_line(i, line))
            }
            (start, end) => {
                let start = match start {
                    Bound::Excluded(n) => n + 1,
                    Bound::Included(n) => *n,
                    Bound::Unbounded => 0,
                };
                Self::line_with_comment(
                    line_index,
                    line,
                    start,
                    match end {
                        Bound::Excluded(n) => n - 1,
                        Bound::Included(n) => *n,
                        Bound::Unbounded => line.chars().count(),
                    }
                    .saturating_sub(start),
                    comment,
                )
            }
        }
    }

    /// Create a context with multiple highlights
    pub fn multiple_highlights(
        line_index: Option<u32>,
        lines: &'text str,
        highlights: impl IntoIterator<Item = (usize, impl RangeBounds<usize>, Option<Cow<'text, str>>)>,
    ) -> Self {
        let lengths = lines.lines().map(|l| l.chars().count()).collect::<Vec<_>>();
        Self {
            source: None,
            line_number: line_index.and_then(|i| NonZeroU32::new(i + 1)),
            lines: lines.into(),
            first_line_offset: 0,
            highlights: highlights
                .into_iter()
                .map(
                    |(line, range, comment)| match (range.start_bound(), range.end_bound()) {
                        (Bound::Unbounded, Bound::Unbounded) => Highlight {
                            line,
                            offset: 0,
                            length: lengths[line],
                            comment,
                        },
                        (start, end) => {
                            let start = match start {
                                Bound::Excluded(n) => n + 1,
                                Bound::Included(n) => *n,
                                Bound::Unbounded => 0,
                            };
                            Highlight {
                                line,
                                offset: start,
                                length: match end {
                                    Bound::Excluded(n) => n - 1,
                                    Bound::Included(n) => *n,
                                    Bound::Unbounded => lengths[line],
                                }
                                .saturating_sub(start),
                                comment,
                            }
                        }
                    },
                )
                .collect(),
        }
    }

    /// Creates a new context to highlight a certain position
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn position(pos: &FilePosition<'_>) -> Self {
        if pos.text.is_empty() {
            Self {
                source: None,
                line_number: NonZeroU32::new(pos.line_index + 1),
                first_line_offset: 0,
                lines: Cow::Borrowed(""),
                highlights: vec![Highlight {
                    line: 0,
                    offset: 0,
                    length: 3,
                    comment: None,
                }],
            }
        } else {
            Self {
                source: None,
                line_number: NonZeroU32::new(pos.line_index + 1),
                first_line_offset: 0,
                lines: Cow::Owned(pos.text.lines().next().unwrap().to_string()),
                highlights: vec![Highlight {
                    line: 0,
                    offset: 0,
                    length: 3,
                    comment: None,
                }],
            }
        }
    }

    /// Creates a new context from a start and end point within a single file
    pub fn range(start: &FilePosition<'text>, end: &FilePosition<'text>) -> Self {
        if start.line_index == end.line_index {
            Self {
                source: None,
                line_number: NonZeroU32::new(start.line_index + 1),
                first_line_offset: start.column,
                lines: Cow::Borrowed(&start.text[..(end.column - start.column) as usize]),
                highlights: vec![Highlight {
                    line: 0,
                    offset: 0,
                    length: (end.column - start.column) as usize,
                    comment: None,
                }],
            }
        } else {
            Self {
                source: None,
                line_number: NonZeroU32::new(start.line_index + 1),
                first_line_offset: start.column,
                lines: Cow::Borrowed(
                    &start.text[..start
                        .text
                        .lines()
                        .take((end.line_index - start.line_index) as usize)
                        .fold(0, |acc, line| acc + line.len() + usize::from(acc != 0))],
                ), // TODO: maybe on windows this might be some bytes off
                highlights: Vec::new(),
            }
        }
    }
}

/// Builder style methods
impl<'text> Context<'text> {
    /// Set the source
    #[must_use]
    pub fn source(self, source: impl Into<Cow<'text, str>>) -> Self {
        Self {
            source: Some(source.into()),
            ..self
        }
    }

    /// Set the line index
    #[must_use]
    pub fn line_index(self, line_index: u32) -> Self {
        Self {
            line_number: NonZeroU32::new(line_index + 1),
            ..self
        }
    }

    /// Set the lines together with the offset of the first line (in characters)
    #[must_use]
    pub fn lines(self, first_line_offset: u32, lines: impl Into<Cow<'text, str>>) -> Self {
        Self {
            first_line_offset,
            lines: lines.into(),
            ..self
        }
    }

    /// Add a highlight
    #[must_use]
    pub fn add_highlight(mut self, highlight: impl Into<Highlight<'text>>) -> Self {
        self.highlights.push(highlight.into());
        self
    }

    /// Add a highlights
    #[must_use]
    pub fn add_highlights<T: Into<Highlight<'text>>>(
        mut self,
        highlights: impl IntoIterator<Item = T>,
    ) -> Self {
        self.highlights
            .extend(highlights.into_iter().map(|i| i.into()));
        self
    }
}

/// Functionality
impl<'text> Context<'text> {
    /// (Possibly) clone the text to get a static valid Context
    pub fn to_owned(self) -> Context<'static> {
        Context {
            source: self.source.map(|c| Cow::Owned(c.into_owned())),
            lines: Cow::Owned(self.lines.into_owned()),
            highlights: self.highlights.into_iter().map(|h| h.to_owned()).collect(),
            ..self
        }
    }

    /// Check if this is an empty context
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.source.is_none() && self.line_number.is_none()
    }

    /// Get the margin needed for the line number (if present)
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    pub(crate) fn margin(&self) -> usize {
        let get_margin = |n| ((n + 1) as f64).log10().max(1.0).ceil() as usize;
        self.line_number.map_or(0, |n| {
            get_margin(n.get() as usize + self.lines.lines().count())
        })
    }

    /// Display this context, with an optional note after the context.
    /// # Errors
    /// If the underlying formatter errors.
    pub(crate) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        note: Option<&str>,
        merged: Merged,
    ) -> fmt::Result {
        #[cfg(not(feature = "ascii-only"))]
        mod symbols {
            pub const HIGHLIGHT_START_LINE: &str = " ╎ ";
            pub const ARC_BOTTOM_TO_RIGHT: char = '╭';
            pub const ARC_TOP_TO_RIGHT: char = '╰';
            pub const LEFT_TO_RIGHT: &str = "─";
            pub const TOP_ENDCAP: char = '╷';
            pub const RIGHT_ENDCAP: char = '╴';
            pub const LEFT_ENDCAP: char = '╶';
            pub const BOTTOM_ENDCAP: char = '╵';
            pub const TOP_TO_BOTTOM: char = '│';
            pub const ELLIPSIS: char = '…';
            pub const LENGTH_ZERO_HIGHLIGHT: char = 'ò';
            pub const LENGTH_ONE_HIGHLIGHT: char = '⁃';
        }
        #[cfg(feature = "ascii-only")]
        mod symbols {
            pub const HIGHLIGHT_START_LINE: &str = " * ";
            pub const ARC_BOTTOM_TO_RIGHT: char = '+';
            pub const ARC_TOP_TO_RIGHT: char = '+';
            pub const LEFT_TO_RIGHT: &str = "-";
            pub const TOP_ENDCAP: char = '.';
            pub const RIGHT_ENDCAP: char = '-';
            pub const LEFT_ENDCAP: char = '-';
            pub const BOTTOM_ENDCAP: char = '\'';
            pub const TOP_TO_BOTTOM: char = '|';
            pub const ELLIPSIS: char = '~';
            pub const LENGTH_ZERO_HIGHLIGHT: char = '^';
            pub const LENGTH_ONE_HIGHLIGHT: char = '-';
        }
        use symbols::*;

        if self.is_empty() {
            Ok(())
        } else if self.lines.is_empty() {
            write!(
                f,
                "[{}{}{}]",
                self.source.as_deref().unwrap_or_default(),
                self.line_number
                    .map(|i| format!(":{i}"))
                    .unwrap_or_default(),
                self.highlights
                    .first()
                    .filter(|h| h.line == 0
                        && self.highlights.len() == 1
                        && self.line_number.is_some())
                    .map(|h| format!(":{}", self.first_line_offset as usize + h.offset + 1))
                    .unwrap_or_default()
            )
        } else {
            let margin = merged.margin().unwrap_or_else(|| self.margin());
            let max_cols: usize = 100 - margin - 3;

            if merged.leading_decoration() {
                if let Some(source) = &self.source {
                    write!(
                        f,
                        "{} {}{source}{}{}{}",
                        " ".repeat(margin),
                        format!("{ARC_BOTTOM_TO_RIGHT}{LEFT_TO_RIGHT}[").blue(),
                        self.line_number
                            .map(|i| format!(":{i}"))
                            .unwrap_or_default(),
                        self.highlights
                            .first()
                            .filter(|h| h.line == 0
                                && self.highlights.len() == 1
                                && self.line_number.is_some())
                            .map(|h| format!(":{}", self.first_line_offset as usize + h.offset + 1))
                            .unwrap_or_default(),
                        ']'.blue(),
                    )?;
                } else {
                    write!(f, "{} {}", " ".repeat(margin), TOP_ENDCAP.blue())?;
                }
            }

            for (index, line) in self.lines.lines().enumerate() {
                let mut highlight_range = None;
                let mut highlights: Vec<_> = self
                    .highlights
                    .iter()
                    .filter(|h| h.line == index)
                    .inspect(|h| {
                        highlight_range = Some(highlight_range.map_or(
                            (h.offset, h.offset.saturating_add(h.length)),
                            |range: (usize, usize)| {
                                (
                                    range.0.min(h.offset),
                                    range.1.max(h.offset.saturating_add(h.length)),
                                )
                            },
                        ));
                    })
                    .collect();
                highlights.sort_by(|a, b| a.offset.cmp(&b.offset));

                let line_length = line.chars().count();
                let displayed_range = highlight_range.filter(|_| line_length > max_cols).map_or(
                    (0, max_cols - 1),
                    |(start, end)| {
                        (
                            start.saturating_sub(5),
                            end.saturating_add(5).min(line_length),
                        )
                    },
                );

                let mut first = true;
                let mut last_line_comment_cut_off = false;
                for start in (displayed_range.0..displayed_range.1).step_by(max_cols - 1) {
                    let end = (start + max_cols).min(line_length); // Absolute position
                    let length = end.saturating_sub(start);

                    write!(
                        f,
                        "\n{:<margin$} {} ",
                        self.line_number
                            .map_or(String::new(), |n| (n.get() as usize + index).to_string())
                            .dimmed(),
                        TOP_TO_BOTTOM.blue(),
                    )?;

                    let front_trimmed =
                        first && (index == 0 && self.first_line_offset > 0) || start != 0;
                    let end_trimmed = end < line_length;
                    if front_trimmed {
                        write!(f, "{ELLIPSIS}")?;
                    }
                    first = false;
                    for c in
                        line.chars().skip(start).take(length.min(
                            max_cols.saturating_sub(
                                usize::from(front_trimmed) + usize::from(end_trimmed),
                            ),
                        ))
                    {
                        #[cfg(not(feature = "ascii-only"))]
                        {
                            write!(
                                f,
                                "{}",
                                match c {
                                    c if c as u32 <= 31 =>
                                        char::try_from(c as u32 + 0x2400).unwrap(),
                                    '\u{007F}' => '␡',
                                    c => c,
                                },
                            )?;
                        }
                        #[cfg(feature = "ascii-only")]
                        {
                            write!(
                                f,
                                "{}",
                                match c {
                                    '\t' => ' ',
                                    '\u{007F}' => '\u{001A}',
                                    c if !c.is_ascii() || c as u32 <= 31 => '\u{001A}',
                                    c => c,
                                },
                            )?;
                        }
                    }
                    if end_trimmed {
                        write!(f, "{ELLIPSIS}")?;
                    }

                    // Display the highlights that are placed on this chunk
                    let mut last_offset: usize = 0; // In absolute offset

                    for high in highlights.iter().filter(|h| {
                        h.offset <= (end - usize::from(front_trimmed) - usize::from(end_trimmed))
                            && h.offset.saturating_add(h.length) >= start
                    }) {
                        // TODO: current layout is not maximally small in number of lines, maybe the highlights could be reordered to place the highest amount of highlights on every line
                        let start_string;
                        let start_offset; // In offset on this line
                        if last_offset != 0 && last_offset <= high.offset {
                            start_string = String::new();
                            start_offset = last_offset;
                        } else {
                            start_string = format!(
                                "\n{}{}{}",
                                " ".repeat(margin),
                                HIGHLIGHT_START_LINE.blue(),
                                if last_line_comment_cut_off {
                                    LEFT_TO_RIGHT
                                } else {
                                    " "
                                }
                                .repeat(usize::from(front_trimmed))
                                .yellow()
                            );
                            start_offset = start + usize::from(front_trimmed);
                            last_line_comment_cut_off = false;
                        }
                        let mut comment_cut_off = false;
                        write!(
                            f,
                            "{start_string}{}{}",
                            " ".repeat(high.offset.saturating_sub(start_offset)),
                            match high.length {
                                0 => LENGTH_ZERO_HIGHLIGHT.to_string(),
                                1 => LENGTH_ONE_HIGHLIGHT.to_string(),
                                n => {
                                    let high_length = high.length.min(line_length - high.offset);
                                    if high.offset < start {
                                        format!(
                                            "{}{RIGHT_ENDCAP}",
                                            LEFT_TO_RIGHT.repeat(
                                                (high.offset + high.length)
                                                    .saturating_sub(start)
                                                    .saturating_sub(1)
                                            )
                                        )
                                    } else if high.offset + high_length
                                        > end - usize::from(end_trimmed)
                                    {
                                        comment_cut_off = true;
                                        last_line_comment_cut_off = true;
                                        format!(
                                            "{LEFT_ENDCAP}{}",
                                            LEFT_TO_RIGHT.repeat(high_length.min(
                                                end - usize::from(end_trimmed)
                                                    - usize::from(front_trimmed)
                                                    - high.offset
                                            ))
                                        )
                                    } else {
                                        format!(
                                            "{LEFT_ENDCAP}{}{RIGHT_ENDCAP}",
                                            LEFT_TO_RIGHT.repeat(
                                                (n - 2).min(
                                                    length
                                                        .saturating_sub(
                                                            high.offset.saturating_sub(start)
                                                        )
                                                        .saturating_sub(2)
                                                )
                                            )
                                        )
                                    }
                                }
                            }
                            .yellow()
                        )?;
                        // Write out the comment
                        if !comment_cut_off {
                            let mut index = high
                                .offset
                                .saturating_sub(start)
                                .saturating_add(high.length);
                            for c in high.comment.as_deref().unwrap_or_default().chars() {
                                if index == max_cols {
                                    index = 0;
                                    write!(
                                        f,
                                        "\n{}{}",
                                        " ".repeat(margin),
                                        HIGHLIGHT_START_LINE.blue()
                                    )?;
                                }
                                write!(f, "{c}")?;
                                index = index.saturating_add(1);
                            }
                            last_offset = index; // TODO: fix
                        }
                        last_offset = high.offset
                            + high
                                .length
                                .max(1)
                                .min(length.saturating_sub(high.offset.saturating_sub(start)))
                            + high.comment.as_ref().map_or(0, |c| c.chars().count())
                            + usize::from(front_trimmed && self.first_line_offset == 0);
                    }
                }
            }
            // Last line
            if merged.trailing_decoration() {
                if let Some(note) = note {
                    write!(
                        f,
                        "\n{:pad$} {}{}{}",
                        "",
                        format!("{ARC_TOP_TO_RIGHT}{LEFT_TO_RIGHT}[").blue(),
                        note,
                        ']'.blue(),
                        pad = margin
                    )?;
                } else {
                    write!(f, "\n{:pad$} {}", "", BOTTOM_ENDCAP.blue(), pad = margin)?;
                }
            }
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Merged {
    No,
    First(usize),
    Middle(usize),
    Last(usize),
}

impl Merged {
    pub(crate) fn leading_decoration(&self) -> bool {
        matches!(self, Self::No | Self::First(_))
    }

    pub(crate) fn trailing_decoration(&self) -> bool {
        matches!(self, Self::No | Self::Last(_))
    }

    pub(crate) fn margin(&self) -> Option<usize> {
        match self {
            Self::First(m) | Self::Middle(m) | Self::Last(m) => Some(*m),
            Self::No => None,
        }
    }
}

impl fmt::Display for Context<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, None, Merged::No)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A position in a file for use in parsing/lexing
pub struct FilePosition<'a> {
    /// The remaining text (as ref so no copies)
    pub text: &'a str,
    /// The current line index
    pub line_index: u32,
    /// The current column number
    pub column: u32,
}

#[cfg(test)]
pub(crate) fn test_characters(text: &str) {
    for c in text.chars() {
        #[cfg(feature = "ascii-only")] // Allow the escape character in ASCII output
        if c == '\u{001A}' {
            continue;
        }
        assert!(
            c == '\n' || (c as u32 > 31 && c != '\u{007F}'),
            "{c} ({}) is invalid range\n{text}",
            c as u32
        );
        #[cfg(feature = "ascii-only")]
        {
            assert!(c.is_ascii(), "{c} is not inside the ASCII range\n{text}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($name:ident: $context:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let context = $context;
                let string = context.to_string();
                #[cfg(not(feature="ascii-only"))]
                if string != $expected {
                    panic!("Generated context:\n{}\nNot identical to expected:\n{}\nThis is the generated string if this actually is correct: {0:?}", string, $expected);
                }
                test_characters(&string);
            }
        };
    }

    test!(empty: Context::none() => "");
    test!(empty_source: Context::default().source("file.txt") => "[file.txt]");
    test!(empty_line: Context::default().line_index(12) => "[:13]");
    test!(empty_line_offset: Context::default().line_index(12).add_highlight((0, 12, 3)) => "[:13:13]");
    test!(empty_source_line_offset: Context::default().source("file.txt").line_index(12).add_highlight((0, 12, 3)) => "[file.txt:13:13]");
    test!(empty_source_offset: Context::default().source("file.txt").add_highlight((0, 12, 3)) => "[file.txt]");
    test!(show: Context::show("Hello world") => " ╷\n │ Hello world\n ╵");
    test!(show_characters: Context::show("Hello world cr\r tab\t null\0") => " ╷\n │ Hello world cr␍ tab␉ null␀\n ╵");
    test!(full_line: Context::full_line(0, "#[derive(Clone, Copy, Debug, Eq, PartialEq)]") 
        => "  ╷\n1 │ #[derive(Clone, Copy, Debug, Eq, PartialEq)]\n  ╵");
    test!(line: Context::line(Some(0), "#[derive(Clone, Copy, Debug, Eq, PartialEq)]", 16, 4) 
        => "  ╷\n1 │ #[derive(Clone, Copy, Debug, Eq, PartialEq)]\n  ╎                 ╶──╴\n  ╵");
    test!(line_range: Context::line_range(Some(0), "\tpub column; usize,", 11..13) 
        => "  ╷\n1 │ ␉pub column; usize,\n  ╎            ⁃\n  ╵");
    test!(line_range_comment: Context::line_range_with_comment(Some(0), "\tpub column; usize,", 11..13, Some(Cow::Borrowed("Use colon instead"))) 
        => "  ╷\n1 │ ␉pub column; usize,\n  ╎            ⁃Use colon instead\n  ╵");
    test!(line_comment: Context::line_with_comment(Some(0), "\tpub column; usize,", 11, 1, Some(Cow::Borrowed("Use colon instead"))) 
        => "  ╷\n1 │ ␉pub column; usize,\n  ╎            ⁃Use colon instead\n  ╵");
    test!(single_line_multiple_highlights: Context::multiple_highlights(Some(0), "0,3\tnull\tmany\t0.0001", [(0, 0..=3, None), (0, 4..=8, None), (0, 9..=13, None)]) 
        => "  ╷\n1 │ 0,3␉null␉many␉0.0001\n  ╎ ╶─╴ ╶──╴ ╶──╴\n  ╵");
    test!(single_line_multiple_highlights_comments: Context::multiple_highlights(Some(0), "0,3\tnull\tmany\t0.0001", [(0, 0..=3, Some(Cow::Borrowed("Score"))), (0, 4..=8, Some(Cow::Borrowed("RT"))), (0, 9..=13, Some(Cow::Borrowed("Method")))]) 
        => "  ╷\n1 │ 0,3␉null␉many␉0.0001\n  ╎ ╶─╴Score\n  ╎     ╶──╴RT\n  ╎          ╶──╴Method\n  ╵");
    test!(builder: Context::default().lines(0, "Hello world").add_highlight((0, 1, 2)).add_highlight((0, 6.., "Rest")) 
        => " ╷\n │ Hello world\n ╎  ╶╴   ╶───╴Rest\n ╵");
    test!(builder_source: Context::default().source("path/file.txt").lines(1, "ello world").add_highlight((0, 0, 2)).add_highlight((0, 5.., "Rest")) 
        => " ╭─[path/file.txt]\n │ …ello world\n ╎  ╶╴   ╶───╴Rest\n ╵");
    test!(builder_source_line_1: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2))
        => "  ╭─[path/file.txt:3:2]\n3 │ …ello world\n  ╎  ╶╴\n  ╵");
    test!(builder_source_line_2: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2)).add_highlight((0, 5.., "Rest")) 
        => "  ╭─[path/file.txt:3]\n3 │ …ello world\n  ╎  ╶╴   ╶───╴Rest\n  ╵");
    test!(builder_line_offset: Context::default().line_index(2).lines(123, "ello world").add_highlight((0, 0, 2)).add_highlight((0, 5.., "Rest")) 
        => "  ╷\n3 │ …ello world\n  ╎  ╶╴   ╶───╴Rest\n  ╵");
    test!(builder_source_line_offset: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2)) 
        => "  ╭─[path/file.txt:3:2]\n3 │ …ello world\n  ╎  ╶╴\n  ╵");
    test!(builder_source_offset: Context::default().source("path/file.txt").lines(1, "ello world").add_highlight((0, 0, 2)) 
        => " ╭─[path/file.txt]\n │ …ello world\n ╎  ╶╴\n ╵");
    test!(multi: Context::default().lines(0, "Hello world\nMake it a good one!") 
        => " ╷\n │ Hello world\n │ Make it a good one!\n ╵");
    test!(multi_highlight_1: Context::default().lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2)).add_highlight((1, 5, 2)).add_highlight((1, 6, 3))
        => " ╷\n │ Hello world\n ╎  ╶╴\n │ Make it a good one!\n ╎      ╶╴\n ╎       ╶─╴\n ╵");
    test!(multi_highlight_2: Context::default().lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2)).add_highlight((1, 5, 2, "Cool")).add_highlight((1, 15, 3, "1"))
        => " ╷\n │ Hello world\n ╎  ╶╴\n │ Make it a good one!\n ╎      ╶╴Cool    ╶─╴1\n ╵");
    test!(multi_source_highlight: Context::default().source("file.txt").lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2))
        => " ╭─[file.txt]\n │ Hello world\n ╎  ╶╴\n │ Make it a good one!\n ╵");
    test!(multi_source_line_highlight: Context::default().source("file.txt").line_index(41).lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2))
        => "   ╭─[file.txt:42:2]\n42 │ Hello world\n   ╎  ╶╴\n43 │ Make it a good one!\n   ╵");
    test!(multi_together: Context::default().source("file.txt").line_index(41).lines(0, "Hello world").add_highlight((0, 1..4)).add_highlight((0, 4..6)).add_highlight((0, 6..7)).add_highlight((0, 7..8))
        => "   ╭─[file.txt:42]\n42 │ Hello world\n   ╎  ╶─╴╶╴⁃⁃\n   ╵");
    test!(csv_try: Context::default().source("file.csv").line_index(1).lines(0, "hihi,  \t\r\t,,1234.56  567,\"hellow,hellow\",rrrr,   rf   ,1,hjksdfhjkfsdhjksdfhkjhjkfsdhjkdsfhjkfdshjksdfhjksfdhjksdjhkfdsjhj")
            .add_highlights([(0, 0..4),(0, 10..10),(0, 11..11),(0, 12..24),(0, 26..39),(0, 41..45),(0, 49..51),(0, 55..56),(0, 57..122)])
        => "  ╭─[file.csv:2]\n2 │ hihi,  ␉␍␉,,1234.56  567,\"hellow,hellow\",rrrr,   rf   ,1,hjksdfhjkfsdhjksdfhkjhjkfsdhjkdsfhjkfd…\n  ╎ ╶──╴      òò╶──────────╴  ╶───────────╴  ╶──╴    ╶╴    ⁃ ╶──────────────────────────────────────\n2 │ …shjksdfhjksfdhjksdjhkfdsjhj\n  ╎ ───────────────────────────╴\n  ╵");
    test!(wrapping_1: Context::default().source("file.csv").line_index(1).lines(0, "saaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabbbbbbbbbbaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaccaaaaaadddddaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .add_highlights([(0, 0..1, "Start"), (0, 90..100, "CommentB"),(0, 183..185, "CommentC"),(0,190..195,"CommentD")])
        => "  ╭─[file.csv:2]\n2 │ saaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabbbbb…\n  ╎ ⁃Start                                                                                    ╶─────\n2 │ …bbbbbaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaccaaaaa…\n  ╎ ─────╴CommentB                                                                          ╶╴Commen\n  ╎ tC\n2 │ …dddddaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n  ╎  ╶───╴CommentD\n  ╵");
    test!(wrapping_2: Context::default().source("file.csv").line_index(1).lines(0, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .add_highlight((0, 0..1, "A very really long comment bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"))
        => "  ╭─[file.csv:2:1]\n2 │ aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa…\n  ╎ ⁃A very really long comment bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n  ╎ bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n  ╎ bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n  ╵");
    test!(wrapping_3: Context::default().source("file.csv").line_index(1).lines(0, "saaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabccccbbbbbaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaccadaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .add_highlights([(0, 0..1, "Start"), (0, 90..100, "CommentB"),(0, 91..95, "CommentC"),(0,183..185,"CommentC"),(0,186..187,"CommentD")])
        => "  ╭─[file.csv:2]\n2 │ saaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabbbbb…\n  ╎ ⁃Start                                                                                    ╶─────\n2 │ …bbbbbaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaccaaaaa…\n  ╎ ─────╴CommentB                                                                          ╶╴Commen\n  ╎ tC\n2 │ …dddddaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n  ╎  ╶───╴CommentD\n  ╵");
}
