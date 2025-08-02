use serde::*;
use std::{
    borrow::Cow,
    fmt,
    ops::{Bound, RangeBounds},
};

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Context<'text> {
    /// The source or path of the text
    source: Option<Cow<'text, str>>,
    /// 0 based index of the first line
    line_index: Option<usize>,
    /// Offset of the first line (in characters) before the slice starts
    first_line_offset: usize,
    /// The text of this context, multiline text is handled by [str::lines]
    lines: Cow<'text, str>,
    /// The highlights, required to be sorted by line first, offset second
    highlights: Vec<Highlight<'text>>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Highlight<'text> {
    /// Line index in case multiple lines are given
    line: usize,
    /// The offset (in chars) into the line
    offset: usize,
    /// The length of the highlight
    length: usize,
    /// Optional comment to post next to the highlight
    comment: Option<Cow<'text, str>>,
}

impl From<(usize, usize, usize)> for Highlight<'static> {
    fn from(value: (usize, usize, usize)) -> Self {
        Self {
            line: value.0,
            offset: value.1,
            length: value.2,
            comment: None,
        }
    }
}

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
                Bound::Excluded(n) => n - offset,
                Bound::Included(n) => n - offset + 1,
                Bound::Unbounded => usize::MAX,
            },
            comment: None,
        }
    }
}

// TODO: used u32 here because otherwise this overlaps with the `(usize, usize, usize)` option
impl<'text, Range: RangeBounds<usize>, Comment: Into<Cow<'text, str>>> From<(u32, Range, Comment)>
    for Highlight<'text>
{
    fn from(value: (u32, Range, Comment)) -> Self {
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

/// Convenience wrappers using common patterns
impl<'text> Context<'text> {
    /// Creates a new context when no context can be given
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates a new context when only a line (eg filename) can be shown
    pub fn show(line: impl Into<Cow<'text, str>>) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_index: None,
            lines: line.into(),
            highlights: Vec::new(),
        }
    }

    /// Creates a new context when a full line is faulty and no special position can be annotated
    pub fn full_line(line_index: usize, line: impl Into<Cow<'text, str>>) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_index: Some(line_index),
            lines: line.into(),
            highlights: Vec::new(),
        }
    }

    /// Creates a new context when a special position can be annotated on a line
    pub fn line(
        line_index: Option<usize>,
        line: impl Into<Cow<'text, str>>,
        offset: usize,
        length: usize,
    ) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_index,
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
        line_index: Option<usize>,
        line: impl Into<Cow<'text, str>>,
        offset: usize,
        length: usize,
        comment: Option<Cow<'text, str>>,
    ) -> Self {
        Self {
            source: None,
            first_line_offset: 0,
            line_index,
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
        line_index: Option<usize>,
        line: &'text str,
        range: impl RangeBounds<usize>,
    ) -> Self {
        Self::line_range_with_comment(line_index, line, range, None)
    }

    /// Create a context highlighting a certain range on a single line
    pub fn line_range_with_comment(
        line_index: Option<usize>,
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
        line_index: Option<usize>,
        lines: &'text str,
        highlights: impl IntoIterator<Item = (usize, impl RangeBounds<usize>, Option<Cow<'text, str>>)>,
    ) -> Self {
        let lengths = lines.lines().map(|l| l.chars().count()).collect::<Vec<_>>();
        Self {
            source: None,
            line_index,
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
    #[expect(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn position(pos: &FilePosition<'_>) -> Self {
        if pos.text.is_empty() {
            Self {
                source: None,
                line_index: Some(pos.line_index),
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
                line_index: Some(pos.line_index),
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
                line_index: Some(start.line_index),
                first_line_offset: start.column,
                lines: Cow::Borrowed(&start.text[..(end.column - start.column)]),
                highlights: vec![Highlight {
                    line: 0,
                    offset: 0,
                    length: end.column - start.column,
                    comment: None,
                }],
            }
        } else {
            Self {
                source: None,
                line_index: Some(start.line_index),
                first_line_offset: start.column,
                lines: Cow::Borrowed(
                    &start.text[..start
                        .text
                        .lines()
                        .take(end.line_index - start.line_index)
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
    pub fn line_index(self, line_index: usize) -> Self {
        Self {
            line_index: Some(line_index),
            ..self
        }
    }

    /// Set the lines together with the offset of the first line (in characters)
    #[must_use]
    pub fn lines(self, first_line_offset: usize, lines: impl Into<Cow<'text, str>>) -> Self {
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
    /// Check if this is an empty context
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Display this context, with an optional note after the context.
    /// # Errors
    /// If the underlying formatter errors.
    fn display(&self, f: &mut fmt::Formatter<'_>, note: Option<&str>) -> fmt::Result {
        const MAX_COLS: usize = 95; // TODO: clip lines if too ling
        const HIGHLIGHT_START_LINE: &str = " ╎ ";

        if self.is_empty() {
            return Ok(());
        }

        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation
        )]
        let get_margin = |n| ((n + 1) as f64).log10().max(1.0).ceil() as usize;
        let margin = self
            .line_index
            .map_or(0, |n| get_margin(n + self.lines.lines().count()));

        if let Some(source) = &self.source {
            write!(
                f,
                "{} ╭─[{source}{}{}]",
                " ".repeat(margin),
                self.line_index
                    .map(|i| format!(":{}", i + 1))
                    .unwrap_or_default(),
                self.highlights
                    .first()
                    .filter(|h| h.line == 0 && self.highlights.len() == 1)
                    .map(|h| format!(":{}", self.first_line_offset + h.offset + 1))
                    .unwrap_or_default() // TODO: think about a different symbol to denote columns as right now :x could be either column or line depending on which one is given
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
                self.line_index
                    .map_or(String::new(), |n| (n + index + 1).to_string()),
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
            let mut last_offset = 0;
            while let Some(high) = highlights_peek.peek() {
                if high.line > index {
                    break;
                }
                if let Some(high) = highlights_peek.next() {
                    // TODO: current layout is not maximally small in number of lines, maybe the highlights could be reordered to place the highest amount of highlights on every line
                    let start;
                    let start_offset;
                    if last_offset != 0 && last_offset < high.offset {
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
                        " ".repeat(high.offset - start_offset),
                        if high.length == 0 {
                            "⏵".to_string()
                        } else {
                            "─".repeat(high.length.min(line_length - high.offset))
                        },
                        high.comment
                            .as_deref()
                            .map_or(String::new(), |c| format!(" {c}")), //Maybe one of: ╸·
                    )?;
                    last_offset = high.offset
                        + high.length.max(1).min(line_length - high.offset)
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
    pub line_index: usize,
    /// The current column number
    pub column: usize,
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
    test!(builder_source_line: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2)).add_highlight((0, 5.., "Rest")) 
        => "  ╭─[path/file.txt:3]\n3 │ …ello world\n  ╎  ──   ───── Rest\n  ╵");
    test!(builder_source_line_offset: Context::default().source("path/file.txt").line_index(2).lines(1, "ello world").add_highlight((0, 0, 2)) 
        => "  ╭─[path/file.txt:3:2]\n3 │ …ello world\n  ╎  ──\n  ╵");
}
