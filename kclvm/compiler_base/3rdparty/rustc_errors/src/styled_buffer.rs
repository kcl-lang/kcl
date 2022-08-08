//! 'StyledBuffer' is responsible for text rendering.

use compiler_base_style::Style;
pub struct StyledBuffer {
    lines: Vec<Vec<StyledChar>>,
}

#[derive(Clone)]
struct StyledChar {
    chr: char,
    style: Option<Box<dyn Style>>,
}

pub struct StyledString {
    pub text: String,
    pub style: Option<Box<dyn Style>>,
}

impl StyledChar {
    const SPACE: StyledChar = StyledChar::new(' ', None);

    const fn new(chr: char, style: Option<Box<dyn Style>>) -> Self {
        StyledChar { chr, style }
    }
}

impl StyledBuffer {
    pub fn new() -> StyledBuffer {
        StyledBuffer { lines: vec![] }
    }

    /// Returns content of `StyledBuffer` split by lines and line styles
    pub fn render(&self) -> Vec<Vec<StyledString>> {
        let mut output: Vec<Vec<StyledString>> = vec![];
        let mut styled_vec: Vec<StyledString> = vec![];

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
    pub fn putc(&mut self, line: usize, col: usize, chr: char, style: Option<Box<dyn Style>>) {
        self.ensure_lines(line);
        if col >= self.lines[line].len() {
            self.lines[line].resize(col + 1, StyledChar::SPACE);
        }
        self.lines[line][col] = StyledChar::new(chr, style);
    }

    /// Sets `string` with `style` for given `line`, starting from `col`.
    /// If `line` does not exist in our buffer, adds empty lines up to the given
    /// and fills the last line with unstyled whitespace.
    pub fn puts(&mut self, line: usize, col: usize, string: &str, style: Option<Box<dyn Style>>) {
        let mut n = col;
        for c in string.chars() {
            self.putc(line, n, c, style.clone());
            n += 1;
        }
    }

    pub fn putl(&mut self, string: &str, style: Option<Box<dyn Style>>) {
        let line = self.num_lines();
        let mut col = 0;
        for c in string.chars() {
            self.putc(line, col, c, style.clone());
            col += 1;
        }
    }

    pub fn appendl(&mut self, string: &str, style: Option<Box<dyn Style>>) {
        let line = if self.num_lines() > 0 {
            self.num_lines() - 1
        } else {
            self.num_lines()
        };
        self.append(line, string, style);
    }

    /// For given `line` inserts `string` with `style` before old content of that line,
    /// adding lines if needed
    pub fn prepend(&mut self, line: usize, string: &str, style: Option<Box<dyn Style>>) {
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
    pub fn append(&mut self, line: usize, string: &str, style: Option<Box<dyn Style>>) {
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
