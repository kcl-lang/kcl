//! 'StyledBuffer', a generic, is responsible for text rendering.
//!
//! An acceptable custom `XXXStyle` for `StyledBuffer` must implement trait `Clone`, `PartialEq`, `Eq` and `Style`.
use crate::Style;

/// An acceptable custom `XXXStyle` for `StyledBuffer` must implement trait `Clone`, `PartialEq`, `Eq` and `Style`.
#[derive(Debug, PartialEq, Eq)]
pub struct StyledBuffer<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    lines: Vec<Vec<StyledChar<T>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StyledChar<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    chr: char,
    style: Option<T>,
}

/// An acceptable custom `XXXStyle` for `StyledString` must implement trait `Clone`, `PartialEq`, `Eq` and `Style`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyledString<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    pub text: String,
    pub style: Option<T>,
}

impl<T> StyledString<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    /// Constructs a new `StyledString` by string and style.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // You need to choose a style for the generic parameter `T` of `StyledString`.
    /// #[derive(Clone, PartialEq, Eq)]
    /// enum MyStyle{
    ///     Style_1
    /// }
    /// impl Style for MyStyle {
    ///     ...
    /// }
    ///
    /// let styled_string = StyledString::<MyStyle>::new("Hello Styled String".to_string(), Some<MyStyle::Style_1>);
    /// ```
    #[inline]
    pub fn new(text: String, style: Option<T>) -> Self {
        StyledString { text, style }
    }
}

impl<T> StyledChar<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    const SPACE: StyledChar<T> = StyledChar::new(' ', None);

    const fn new(chr: char, style: Option<T>) -> Self {
        StyledChar { chr, style }
    }
}

impl<T> StyledBuffer<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    pub fn new() -> StyledBuffer<T> {
        StyledBuffer { lines: vec![] }
    }

    /// Returns content of `StyledBuffer` split by lines and line styles
    pub fn render(&self) -> Vec<Vec<StyledString<T>>> {
        let mut output: Vec<Vec<StyledString<T>>> = vec![];
        let mut styled_vec: Vec<StyledString<T>> = vec![];

        for styled_line in &self.lines {
            let mut current_style = None;
            let mut current_text = String::new();

            for sc in styled_line {
                if sc.style != current_style {
                    if !current_text.is_empty() {
                        styled_vec.push(StyledString {
                            text: current_text,
                            style: current_style,
                        });
                    }
                    current_style = sc.style.clone();
                    current_text = String::new();
                }
                current_text.push(sc.chr);
            }
            if !current_text.is_empty() {
                styled_vec.push(StyledString {
                    text: current_text,
                    style: current_style,
                });
            }

            // done with the row, push and keep going
            output.push(styled_vec);

            styled_vec = vec![];
        }

        output
    }

    fn ensure_lines(&mut self, line: usize) {
        if line >= self.lines.len() {
            self.lines.resize(line + 1, Vec::new());
        }
    }

    /// Sets `chr` with `style` for given `line`, `col`.
    /// If `line` does not exist in our buffer, adds empty lines up to the given
    /// and fills the last line with unstyled whitespace.
    pub fn putc(&mut self, line: usize, col: usize, chr: char, style: Option<T>) {
        self.ensure_lines(line);
        if col >= self.lines[line].len() {
            self.lines[line].resize(col + 1, StyledChar::SPACE);
        }
        self.lines[line][col] = StyledChar::new(chr, style);
    }

    /// Sets `string` with `style` for given `line`, starting from `col`.
    /// If `line` does not exist in our buffer, adds empty lines up to the given
    /// and fills the last line with unstyled whitespace.
    pub fn puts(&mut self, line: usize, col: usize, string: &str, style: Option<T>) {
        let mut n = col;
        for c in string.chars() {
            self.putc(line, n, c, style.clone());
            n += 1;
        }
    }

    /// Sets `string` with `style` for a new line, starting from col 0.
    /// It will add an new empty line after all the buffer lines for the `string`.
    pub fn pushs(&mut self, string: &str, style: Option<T>) {
        let line = self.num_lines();
        let mut col = 0;
        for c in string.chars() {
            self.putc(line, col, c, style.clone());
            col += 1;
        }
    }

    /// For the last line inserts `string` with `style` after old content of that line,
    /// adding a new line if the `StyledBuffer` has no line.
    pub fn appendl(&mut self, string: &str, style: Option<T>) {
        let line = if self.num_lines() > 0 {
            self.num_lines() - 1
        } else {
            self.num_lines()
        };
        self.append(line, string, style);
    }

    /// For given `line` inserts `string` with `style` before old content of that line,
    /// adding lines if needed
    pub fn prepend(&mut self, line: usize, string: &str, style: Option<T>) {
        self.ensure_lines(line);
        let string_len = string.chars().count();

        if !self.lines[line].is_empty() {
            // Push the old content over to make room for new content
            for _ in 0..string_len {
                self.lines[line].insert(0, StyledChar::SPACE);
            }
        }

        self.puts(line, 0, string, style);
    }

    /// For given `line` inserts `string` with `style` after old content of that line,
    /// adding lines if needed
    pub fn append(&mut self, line: usize, string: &str, style: Option<T>) {
        if line >= self.lines.len() {
            self.puts(line, 0, string, style);
        } else {
            let col = self.lines[line].len();
            self.puts(line, col, string, style);
        }
    }

    pub fn num_lines(&self) -> usize {
        self.lines.len()
    }
}
