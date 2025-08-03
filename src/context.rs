use std::{
    borrow::Cow,
    fmt,
    num::NonZeroU32,
    ops::{Bound, RangeBounds},
};

use crate::Highlight;

/// A context construct to indicate a context presumably in a file, but could be in any kind of source text
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Context<'text> {
    /// The source or path of the text
    source: Option<Cow<'text, str>>,
    /// 1 based index of the first line (0 is used as niche for the None case)
    line_number: Option<NonZeroU32>,
    /// Offset of the first line (in characters) before the slice starts
    first_line_offset: u32,
    /// The text of this context, multiline text is handled by [str::lines]
    lines: Cow<'text, str>,
    /// The highlights, required to be sorted by line first, offset second
    highlights: Vec<Highlight<'text>>,
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
        offset: u32,
        length: u8,
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
        offset: u32,
        length: u8,
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
        range: impl RangeBounds<u32>,
    ) -> Self {
        Self::line_range_with_comment(line_index, line, range, None)
    }

    /// Create a context highlighting a certain range on a single line
    pub fn line_range_with_comment(
        line_index: Option<u32>,
        line: &'text str,
        range: impl RangeBounds<u32>,
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
                    u8::try_from(
                        match end {
                            Bound::Excluded(n) => n - 1,
                            Bound::Included(n) => *n,
                            Bound::Unbounded => {
                                u32::try_from(line.chars().count()).unwrap_or(u32::MAX)
                            }
                        }
                        .saturating_sub(start),
                    )
                    .unwrap_or(u8::MAX),
                    comment,
                )
            }
        }
    }

    /// Create a context with multiple highlights
    pub fn multiple_highlights(
        line_index: Option<u32>,
        lines: &'text str,
        highlights: impl IntoIterator<Item = (u8, impl RangeBounds<u32>, Option<Cow<'text, str>>)>,
    ) -> Self {
        let lengths = lines
            .lines()
            .map(|l| u32::try_from(l.chars().count()).unwrap_or(u32::MAX))
            .collect::<Vec<_>>();
        Self {
            source: None,
            line_number: line_index.and_then(|i| NonZeroU32::new(i + 1)),
            lines: lines.into(),
            first_line_offset: 0,
            // TODO: sort highlights (could this be the place to do placement optimisation?)
            highlights: highlights
                .into_iter()
                .map(
                    |(line, range, comment)| match (range.start_bound(), range.end_bound()) {
                        (Bound::Unbounded, Bound::Unbounded) => Highlight {
                            line,
                            offset: 0,
                            length: u8::try_from(lengths[line as usize]).unwrap_or(u8::MAX),
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
                                length: u8::try_from(
                                    match end {
                                        Bound::Excluded(n) => n - 1,
                                        Bound::Included(n) => *n,
                                        Bound::Unbounded => lengths[line as usize],
                                    }
                                    .saturating_sub(start),
                                )
                                .unwrap_or(u8::MAX),
                                comment,
                            }
                        }
                    },
                )
                .collect(),
        }
    }

    /// Creates a new context to highlight a certain position
    #[expect(clippy::unwrap_used, clippy::missing_panics_doc)]
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
                    length: u8::try_from(end.column - start.column).unwrap_or(u8::MAX),
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
        // TODO: keep sorted
        self.highlights.push(highlight.into());
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

    /// Display this context, with an optional note after the context.
    /// # Errors
    /// If the underlying formatter errors.
    fn display(&self, f: &mut fmt::Formatter<'_>, note: Option<&str>) -> fmt::Result {
        const MAX_COLS: usize = 95; // TODO: clip lines if too ling
        const HIGHLIGHT_START_LINE: &str = " ╎ ";

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
                    .map(|h| format!(":{}", self.first_line_offset + h.offset + 1))
                    .unwrap_or_default()
            )
        } else {
            #[expect(
                clippy::cast_sign_loss,
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation
            )]
            let get_margin = |n| ((n + 1) as f64).log10().max(1.0).ceil() as usize;
            let margin = self.line_number.map_or(0, |n| {
                get_margin(n.get() as usize + self.lines.lines().count())
            });

            if let Some(source) = &self.source {
                write!(
                    f,
                    "{} ╭─[{source}{}{}]",
                    " ".repeat(margin),
                    self.line_number
                        .map(|i| format!(":{i}"))
                        .unwrap_or_default(),
                    self.highlights
                        .first()
                        .filter(|h| h.line == 0
                            && self.highlights.len() == 1
                            && self.line_number.is_some())
                        .map(|h| format!(":{}", self.first_line_offset + h.offset + 1))
                        .unwrap_or_default()
                )?;
            } else {
                write!(f, "{} ╷", " ".repeat(margin))?;
            }
            let mut highlights_peek = self.highlights.iter().peekable();

            for (index, line) in self.lines.lines().enumerate() {
                let front_trimmed = index == 0 && self.first_line_offset > 0;
                write!(
                    f,
                    "\n{:<margin$} │ ",
                    self.line_number
                        .map_or(String::new(), |n| (n.get() as usize + index).to_string()),
                )?;
                if front_trimmed {
                    write!(f, "…")?;
                }
                let mut line_length = 0;
                // TODO: get highlights to check if the line can be truncated
                for c in line.chars() {
                    write!(
                        f,
                        "{}",
                        match c {
                            c if c as u32 <= 31 => char::try_from(c as u32 + 0x2400).unwrap(),
                            '\u{007F}' => '␡',
                            c => c,
                        }
                    )?;
                    line_length += 1;
                }
                let mut last_offset: usize = 0;
                while let Some(high) = highlights_peek.peek() {
                    if high.line as usize > index {
                        break;
                    }
                    if let Some(high) = highlights_peek.next() {
                        // TODO: current layout is not maximally small in number of lines, maybe the highlights could be reordered to place the highest amount of highlights on every line
                        let start;
                        let start_offset;
                        if last_offset != 0 && last_offset < high.offset as usize {
                            start = String::new();
                            start_offset = last_offset;
                        } else {
                            start = format!(
                                "\n{}{HIGHLIGHT_START_LINE}{}",
                                " ".repeat(margin),
                                " ".repeat(usize::from(front_trimmed))
                            );
                            start_offset = 0;
                        }
                        write!(
                            f,
                            "{start}{}{}{}",
                            " ".repeat(high.offset as usize - start_offset),
                            if high.length == 0 {
                                "⏵".to_string()
                            } else {
                                "─"
                                    .repeat((high.length as u32).min(line_length - high.offset)
                                        as usize)
                            },
                            high.comment
                                .as_deref()
                                .map_or(String::new(), |c| format!(" {c}")), //Maybe one of: ╸·
                        )?;
                        last_offset = high.offset as usize
                            + usize::from(high.length)
                                .max(1)
                                .min((line_length - high.offset) as usize)
                            + high.comment.as_ref().map_or(0, |c| 1 + c.chars().count());
                    }
                }
            }
            // Last line
            if let Some(note) = note {
                write!(f, "\n{:pad$} ╰─[{}]", "", note, pad = margin)
            } else {
                write!(f, "\n{:pad$} ╵", "", pad = margin)
            }
        }
    }
}

impl fmt::Display for Context<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, None)
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
mod tests {
    use super::*;

    macro_rules! test {
        ($name:ident: $context:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let context = $context;
                let string = context.to_string();
                if string != $expected {
                    panic!("Generated context:\n{}\nNot identical to expected:\n{}\nThis is the generated if this actually is correct: {0:?}", string, $expected);
                }
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
        => "  ╷\n1 │ #[derive(Clone, Copy, Debug, Eq, PartialEq)]\n  ╎                 ────\n  ╵");
    test!(line_range: Context::line_range(Some(0), "\tpub column; usize,", 11..13) 
        => "  ╷\n1 │ ␉pub column; usize,\n  ╎            ─\n  ╵");
    test!(line_range_comment: Context::line_range_with_comment(Some(0), "\tpub column; usize,", 11..13, Some(Cow::Borrowed("Use colon instead"))) 
        => "  ╷\n1 │ ␉pub column; usize,\n  ╎            ─ Use colon instead\n  ╵");
    test!(line_comment: Context::line_with_comment(Some(0), "\tpub column; usize,", 11, 1, Some(Cow::Borrowed("Use colon instead"))) 
        => "  ╷\n1 │ ␉pub column; usize,\n  ╎            ─ Use colon instead\n  ╵");
    test!(single_line_multiple_highlights: Context::multiple_highlights(Some(0), "0,3\tnull\tmany\t0.0001", [(0, 0..=3, None), (0, 4..=8, None), (0, 9..=13, None)]) 
        => "  ╷\n1 │ 0,3␉null␉many␉0.0001\n  ╎ ─── ──── ────\n  ╵");
    test!(single_line_multiple_highlights_comments: Context::multiple_highlights(Some(0), "0,3\tnull\tmany\t0.0001", [(0, 0..=3, Some(Cow::Borrowed("Score"))), (0, 4..=8, Some(Cow::Borrowed("RT"))), (0, 9..=13, Some(Cow::Borrowed("Method")))]) 
        => "  ╷\n1 │ 0,3␉null␉many␉0.0001\n  ╎ ─── Score\n  ╎     ──── RT\n  ╎          ──── Method\n  ╵");
    test!(builder: Context::default().lines(0, "Hello world").add_highlight((0, 1, 2)).add_highlight((0, 6.., "Rest")) 
        => " ╷\n │ Hello world\n ╎  ──   ───── Rest\n ╵");
    test!(builder_source: Context::default().source("path/file.txt").lines(1, "ello world").add_highlight((0, 0, 2)).add_highlight((0, 5.., "Rest")) 
        => " ╭─[path/file.txt]\n │ …ello world\n ╎  ──   ───── Rest\n ╵");
    test!(builder_source_line_1: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2))
        => "  ╭─[path/file.txt:3:2]\n3 │ …ello world\n  ╎  ──\n  ╵");
    test!(builder_source_line_2: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2)).add_highlight((0, 5.., "Rest")) 
        => "  ╭─[path/file.txt:3]\n3 │ …ello world\n  ╎  ──   ───── Rest\n  ╵");
    test!(builder_source_line_offset: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2)) 
        => "  ╭─[path/file.txt:3:2]\n3 │ …ello world\n  ╎  ──\n  ╵");
    test!(builder_source_offset: Context::default().source("path/file.txt").lines(1, "ello world").add_highlight((0, 0, 2)) 
        => " ╭─[path/file.txt]\n │ …ello world\n ╎  ──\n ╵");
    test!(multi: Context::default().lines(0, "Hello world\nMake it a good one!") 
        => " ╷\n │ Hello world\n │ Make it a good one!\n ╵");
    test!(multi_highlight_1: Context::default().lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2)).add_highlight((1, 5, 2)).add_highlight((1, 6, 3))
        => " ╷\n │ Hello world\n ╎  ──\n │ Make it a good one!\n ╎      ──\n ╎       ───\n ╵");
    test!(multi_highlight_2: Context::default().lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2)).add_highlight((1, 5, 2, "Cool")).add_highlight((1, 15, 3, "1"))
        => " ╷\n │ Hello world\n ╎  ──\n │ Make it a good one!\n ╎      ── Cool   ─── 1\n ╵");
    test!(multi_source_highlight: Context::default().source("file.txt").lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2))
        => " ╭─[file.txt]\n │ Hello world\n ╎  ──\n │ Make it a good one!\n ╵");
    test!(multi_source_line_highlight: Context::default().source("file.txt").line_index(41).lines(0, "Hello world\nMake it a good one!").add_highlight((0, 1, 2))
        => "   ╭─[file.txt:42:2]\n42 │ Hello world\n   ╎  ──\n43 │ Make it a good one!\n   ╵");
}
