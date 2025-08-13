#[cfg(feature = "colored")]
use colored;

pub(crate) trait Coloured {
    type Output;
    fn blue(self) -> Self::Output;
    fn yellow(self) -> Self::Output;
    fn red(self) -> Self::Output;
    fn green(self) -> Self::Output;
    fn dimmed(self) -> Self::Output;
}

#[cfg(not(feature = "colored"))]
impl Coloured for String {
    type Output = String;
    fn blue(self) -> Self::Output {
        self
    }
    fn yellow(self) -> Self::Output {
        self
    }
    fn red(self) -> Self::Output {
        self
    }
    fn green(self) -> Self::Output {
        self
    }
    fn dimmed(self) -> Self::Output {
        self
    }
}

#[cfg(feature = "colored")]
impl Coloured for String {
    type Output = colored::ColoredString;
    fn blue(self) -> Self::Output {
        colored::Colorize::blue(self.as_str())
    }
    fn yellow(self) -> Self::Output {
        colored::Colorize::yellow(self.as_str())
    }
    fn red(self) -> Self::Output {
        colored::Colorize::red(self.as_str())
    }
    fn green(self) -> Self::Output {
        colored::Colorize::green(self.as_str())
    }
    fn dimmed(self) -> Self::Output {
        colored::Colorize::dimmed(self.as_str())
    }
}

#[cfg(not(feature = "colored"))]
impl<'a> Coloured for &'a str {
    type Output = &'a str;
    fn blue(self) -> Self::Output {
        self
    }
    fn yellow(self) -> Self::Output {
        self
    }
    fn red(self) -> Self::Output {
        self
    }
    fn green(self) -> Self::Output {
        self
    }
    fn dimmed(self) -> Self::Output {
        self
    }
}

#[cfg(feature = "colored")]
impl Coloured for &str {
    type Output = colored::ColoredString;
    fn blue(self) -> Self::Output {
        colored::Colorize::blue(self)
    }
    fn yellow(self) -> Self::Output {
        colored::Colorize::yellow(self)
    }
    fn red(self) -> Self::Output {
        colored::Colorize::red(self)
    }
    fn green(self) -> Self::Output {
        colored::Colorize::green(self)
    }
    fn dimmed(self) -> Self::Output {
        colored::Colorize::dimmed(self)
    }
}

#[cfg(not(feature = "colored"))]
impl Coloured for char {
    type Output = char;
    fn blue(self) -> Self::Output {
        self
    }
    fn yellow(self) -> Self::Output {
        self
    }
    fn red(self) -> Self::Output {
        self
    }
    fn green(self) -> Self::Output {
        self
    }
    fn dimmed(self) -> Self::Output {
        self
    }
}

#[cfg(feature = "colored")]
impl Coloured for char {
    type Output = colored::ColoredString;
    fn blue(self) -> Self::Output {
        colored::Colorize::blue(self.to_string().as_str())
    }
    fn yellow(self) -> Self::Output {
        colored::Colorize::yellow(self.to_string().as_str())
    }
    fn red(self) -> Self::Output {
        colored::Colorize::red(self.to_string().as_str())
    }
    fn green(self) -> Self::Output {
        colored::Colorize::green(self.to_string().as_str())
    }
    fn dimmed(self) -> Self::Output {
        colored::Colorize::dimmed(self.to_string().as_str())
    }
}
