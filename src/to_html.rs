use core::fmt;

use crate::{BoxedError, Context, CustomError};

pub trait ToHtml {
    fn display_html(&self, f: &mut impl fmt::Write) -> fmt::Result;
    fn to_html(&self) -> String {
        let mut string = String::new();
        self.display_html(&mut string)
            .expect("Errored while writing to string");
        string
    }
}

impl ToHtml for BoxedError<'_> {
    fn display_html(&self, f: &mut impl fmt::Write) -> fmt::Result {
        self.content.display_html(f)
    }
}

impl ToHtml for CustomError<'_> {
    fn display_html(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(
            f,
            "<div class='{}'>",
            if self.warning { "warning" } else { "error" },
        )?;

        write!(f, "<p class='title'>{}</p>", self.short_description)?;

        write!(f, "<div class='contexts'>")?;
        for context in &self.contexts {
            context.display_html(f)?;
        }
        write!(f, "</div>")?;

        write!(f, "<p class='description'>{}</p>", self.long_description)?;
        if !self.suggestions.is_empty() {
            write!(
                f,
                "<p>Did you mean{}?</p><ul>",
                if self.suggestions.len() == 1 {
                    ""
                } else {
                    " any of"
                }
            )?;
            for suggestion in &self.suggestions {
                write!(f, "<li class='suggestion'>{suggestion}</li>")?;
            }
            write!(f, "</ul>")?;
        }
        if !self.version.is_empty() {
            write!(
                f,
                "<p class='version'>Version: <span class='version-text'>{}</span></p>",
                self.version
            )?;
        }
        if !self.underlying_errors.is_empty() {
            write!(
                f,
                "<label><input type='checkbox'></input> Underlying error{}</label><ul>",
                if self.suggestions.len() == 1 { "" } else { "s" }
            )?;
            for error in &self.underlying_errors {
                write!(f, "<li class='underlying_error'>")?;
                error.display_html(f)?;
                write!(f, "</li>")?;
            }
            write!(f, "</ul>")?;
        }

        write!(f, "</div>",)?;
        Ok(())
    }
}

impl ToHtml for Context<'_> {
    fn display_html(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if self.is_empty() {
            Ok(())
        } else if self.lines.is_empty() {
            write!(f, "<div class='context'>")?;
            write!(
                f,
                "<span class='source'>{}{}{}</span>",
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
            )?;
            write!(f, "</div>")?;
            Ok(())
        } else {
            write!(f, "<div class='context'>")?;
            if let Some(source) = &self.source {
                write!(
                    f,
                    "<span class='source'>{source}{}{}</span>",
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
                )?;
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
                let max_cols = 195;

                let line_length = line.chars().count();
                let displayed_range = highlight_range.filter(|_| line_length > max_cols).map_or(
                    (0, max_cols - 1),
                    |(start, end)| {
                        (
                            start.saturating_sub(5),
                            end.saturating_add(5)
                                .min(line_length)
                                .min(start.saturating_sub(5) + max_cols),
                        )
                    },
                );

                write!(
                    f,
                    "<span class='line-number'>{}</span><span class='line'>",
                    self.line_number
                        .map_or(String::new(), |n| (n.get() as usize + index).to_string())
                )?;

                if displayed_range.0 != 0 {
                    write!(f, "…")?;
                }

                for (char_index, c) in line
                    .chars()
                    .enumerate()
                    .skip(displayed_range.0)
                    .take(displayed_range.1 - displayed_range.0)
                {
                    for high in &highlights {
                        if high.offset == char_index {
                            write!(
                                f,
                                "<span class='highlight' title='{}'>",
                                high.comment.as_deref().unwrap_or_default()
                            )?;
                        }
                    }
                    write!(f, "{c}")?;
                    for high in &highlights {
                        if high.offset + high.length == char_index {
                            write!(f, "</span>")?;
                        }
                    }
                }

                if displayed_range.1 != line_length {
                    write!(f, "…")?;
                }

                write!(f, "</span>")?;
            }
            write!(f, "</div>")?;
            Ok(())
        }
    }
}
