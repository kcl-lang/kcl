use kcl_ast::{
    ast::{self, Module},
    token::TokenKind,
    walker::MutSelfTypedResultWalker,
};
use kcl_primitives::IndexMap;
use std::collections::VecDeque;
mod node;

#[cfg(test)]
mod tests;

pub const WHITESPACE: &str = " ";
pub const TAB: &str = "\t";
pub const NEWLINE: &str = "\n";

#[derive(Debug, Clone)]
pub enum Indentation {
    Indent = 0,
    Dedent = 1,
    Newline = 2,
    IndentWithNewline = 3,
    DedentWithNewline = 4,
    Fill = 5,
}

/// Printer config
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Display width of a TAB character. Per the EditorConfig spec, this is a
    /// *display* hint only — it does not change the bytes the printer emits
    /// for a tab-indented line. In space-mode, `tab_len` is unused.
    pub tab_len: usize,
    /// Width of a single indent level (in spaces when `use_spaces` is true).
    pub indent_len: usize,
    /// `true` emits spaces (controlled by `indent_len`); `false` emits a
    /// single TAB per indent level regardless of `indent_len`/`tab_len`.
    pub use_spaces: bool,
    /// Whether to emit source comments in the printed output.
    pub write_comments: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_len: 4,
            indent_len: 4,
            use_spaces: true,
            write_comments: true,
        }
    }
}

#[derive(Copy, Clone)]
pub struct NoHook;

impl PrinterHook for NoHook {}

pub enum ASTNode<'p> {
    Stmt(&'p ast::NodeRef<ast::Stmt>),
    Expr(&'p ast::NodeRef<ast::Expr>),
}

pub trait PrinterHook {
    fn pre(&self, _printer: &mut Printer<'_>, _node: ASTNode<'_>) {}
    fn post(&self, _printer: &mut Printer<'_>, _node: ASTNode<'_>) {}
}

pub struct Printer<'p> {
    /// Output string buffer.
    pub out: String,
    /// Temporary buffers for look ahead formatting
    pub tmp_buffers: Vec<String>,
    pub indent: usize,
    pub cfg: Config,
    /// Print comments,
    pub comments: VecDeque<ast::NodeRef<ast::Comment>>,
    pub import_spec: IndexMap<String, String>,
    pub hook: &'p (dyn PrinterHook + 'p),
    /// Last AST expr/stmt line, default is 0.
    last_ast_line: u64,
    /// Stack of the current expression span (start_line, end_line).
    expr_span_stack: Vec<(u64, u64)>,
}

impl Default for Printer<'_> {
    fn default() -> Self {
        Self {
            hook: &NoHook,
            out: Default::default(),
            tmp_buffers: Vec::default(),
            indent: Default::default(),
            cfg: Default::default(),
            comments: Default::default(),
            import_spec: Default::default(),
            last_ast_line: Default::default(),
            expr_span_stack: Default::default(),
        }
    }
}

impl<'p> Printer<'p> {
    pub fn new(cfg: Config, hook: &'p (dyn PrinterHook + 'p)) -> Self {
        Self {
            out: "".to_string(),
            tmp_buffers: Vec::default(),
            indent: 0,
            cfg,
            comments: VecDeque::default(),
            import_spec: IndexMap::default(),
            hook,
            last_ast_line: 0,
            expr_span_stack: Vec::default(),
        }
    }

    // --------------------------
    // Write functions
    // --------------------------

    /// Write a string
    #[inline]
    pub fn write(&mut self, text: &str) {
        self.write_string(text);
    }

    /// Write a string with newline.
    #[inline]
    pub fn writeln(&mut self, text: &str) {
        self.write_string(text);
        self.write_string(NEWLINE);
        self.fill("");
    }

    /// Write a space.
    #[inline]
    pub fn write_space(&mut self) {
        self.write_string(WHITESPACE);
    }

    /// Fill a indent
    pub fn fill(&mut self, text: &str) {
        if self.cfg.use_spaces {
            self.write(&format!(
                "{}{}",
                WHITESPACE.repeat(self.indent * self.cfg.indent_len),
                text
            ));
        } else {
            self.write(&format!("{}{}", TAB.repeat(self.indent), text));
        }
    }

    /// Print string
    #[inline]
    pub fn write_string(&mut self, string: &str) {
        self.tmp_buffers
            .last_mut()
            .unwrap_or(&mut self.out)
            .push_str(string);
    }

    pub fn write_indentation(&mut self, indentation: Indentation) {
        match indentation {
            Indentation::Indent => self.enter(),
            Indentation::Dedent => self.leave(),
            Indentation::Newline => self.write_newline(),
            Indentation::IndentWithNewline => {
                self.enter();
                self.write_newline()
            }
            Indentation::DedentWithNewline => {
                self.leave();
                self.write_newline();
            }
            Indentation::Fill => self.fill(""),
        }
    }

    #[inline]
    pub fn write_newline(&mut self) {
        self.writeln("")
    }

    #[inline]
    pub fn write_newline_without_fill(&mut self) {
        self.write_string(NEWLINE);
    }

    /// Print value
    #[inline]
    pub fn write_value<T: std::fmt::Display>(&mut self, value: T) {
        self.write(&format!("{}", value));
    }

    /// Print ast token
    #[inline]
    pub fn write_token(&mut self, tok: TokenKind) {
        let tok_str: String = tok.into();
        self.write_string(&tok_str);
    }

    /// Print ast node
    #[inline]
    pub fn write_node(&mut self, node: ASTNode<'_>) {
        match node {
            ASTNode::Stmt(stmt) => self.stmt(stmt),
            ASTNode::Expr(expr) => self.expr(expr),
        }
    }

    /// Print ast module.
    #[inline]
    pub fn write_module(&mut self, module: &ast::Module) {
        self.walk_module(module);
        while let Some(comment) = self.comments.pop_front() {
            self.writeln(&comment.node.text);
            self.fill("");
        }
    }

    /// Wether has comments on ast node.
    pub(crate) fn has_comments_on_node<T>(&mut self, node: &ast::NodeRef<T>) -> bool {
        if !self.cfg.write_comments {
            return false;
        }
        let mut index = None;
        for (i, comment) in self.comments.iter().enumerate() {
            if comment.line <= node.line {
                index = Some(i);
            } else {
                break;
            }
        }
        index.is_some()
    }

    /// Print ast comments.
    pub fn update_last_ast_line<T>(&mut self, node: &ast::NodeRef<T>) {
        if node.line > self.last_ast_line {
            self.last_ast_line = node.line;
        }
    }

    /// Print ast comments.
    pub fn write_comments_before_node<T>(&mut self, node: &ast::NodeRef<T>) {
        if !self.cfg.write_comments {
            return;
        }
        if node.line > self.last_ast_line {
            self.last_ast_line = node.line;
            let mut index = None;
            for (i, comment) in self.comments.iter().enumerate() {
                if comment.line > node.line {
                    break;
                }
                if comment.line < node.line {
                    // Strictly before this node — emit as a leading comment.
                    index = Some(i);
                } else {
                    // Same line as the node. A same-line leading comment
                    // (`# foo\nbar = 1` where `# foo` is positioned before
                    // `bar`) is still a leading comment, so emit it. An
                    // inline trailing comment (`bar = 1  # foo`) starts
                    // after the node's column and must be emitted later
                    // by `write_inline_trailing_comments`; leave it in
                    // the queue here.
                    if comment.column <= node.column {
                        index = Some(i);
                    }
                }
            }
            if let Some(index) = index {
                let mut count = index as isize;
                while count >= 0 {
                    match self.comments.pop_front() {
                        Some(comment) => {
                            self.writeln(&comment.node.text);
                            if let Some(next_comment) = self.comments.front()
                                && next_comment.line >= comment.line + 2
                                && count > 0
                            {
                                self.write_newline();
                            }
                        }
                        None => break,
                    }
                    count -= 1;
                }
            }
        }
    }

    /// Emit any pending inline trailing comments for `node`.
    ///
    /// An inline trailing comment is a comment that shares the same line
    /// as the node's end and starts at or after the node's end column,
    /// e.g. `bar: int  # comment` or `}# comment`. These are skipped by
    /// [`Self::write_comments_before_node`] (so they are not popped into
    /// a leading comment) and emitted here on the same line as the node
    /// content, preserving the original source layout from issue
    /// kcl-lang/kcl#1756.
    ///
    /// Note on the column comparison: the parser reports `end_column` as
    /// the column where the *next* non-whitespace token begins — not
    /// strictly one past the last byte of the node. So when the user
    /// writes `value# comment` (no separating space), `comment.column`
    /// equals `node.end_column` and we still need to treat the comment
    /// as inline trailing. Hence the `>=` rather than `>`.
    ///
    /// This helper is tolerant of being called either before or after
    /// the trailing newline that ends the node's line has been emitted.
    /// If the buffer currently ends with `\n`, that newline is popped so
    /// the comment lands on the same line as the node; after all trailing
    /// comments have been emitted, the newline is restored. Callers do
    /// not need to know the exact state of the buffer.
    pub fn write_inline_trailing_comments<T>(&mut self, node: &ast::NodeRef<T>) {
        if !self.cfg.write_comments {
            return;
        }
        if node.end_line == 0 {
            return;
        }
        // Count any trailing newlines at the end of the buffer so we can
        // restore them after popping one for the comment. There may be
        // more than one (e.g. a blank line between two statements), in
        // which case the comment belongs on the *node's* line, not the
        // blank one — so we restore all of them except the first.
        let trailing_newlines = self.out.len() - self.out.trim_end_matches('\n').len();
        let had_trailing_newline = trailing_newlines > 0;
        let mut wrote_any = false;
        while let Some(comment) = self.comments.front() {
            // A trailing comment for this node lives either on the same
            // line as the node's *end* (e.g. `bar: int  # foo`) or on
            // the same line as the node's *start* (e.g.
            // `config = { # foo` where `{` opens a multi-line value).
            let on_start_line = comment.line == node.line;
            let on_end_line = comment.line == node.end_line;
            if !on_start_line && !on_end_line {
                break;
            }
            if comment.column < node.end_column {
                break;
            }
            let comment = self.comments.pop_front().unwrap();
            if !wrote_any && had_trailing_newline {
                for _ in 1..trailing_newlines {
                    self.out.pop();
                }
                self.out.pop();
            }
            self.write_space();
            self.write(&comment.node.text);
            wrote_any = true;
        }
        if wrote_any && had_trailing_newline {
            // Restore all but one of the popped newlines so any blank
            // line(s) the caller inserted between stmts are preserved.
            for _ in 1..trailing_newlines {
                self.write_string(NEWLINE);
            }
            self.write_string(NEWLINE);
        }
    }

    pub fn write_comments_until_line(&mut self, line: u64) {
        if !self.cfg.write_comments || line == 0 {
            return;
        }

        let mut comments_written = 0;
        while let Some(comment) = self.comments.front() {
            if comment.line == 0 || comment.line > line {
                break;
            }

            let comment = self.comments.pop_front().unwrap();

            // Add newline before first comment if not already at line start
            if comments_written == 0 && !self.out.ends_with('\n') {
                self.write_newline_without_fill();
            }

            self.fill("");
            self.write(&comment.node.text);
            self.write_newline_without_fill();

            // Add extra newline if next comment is 2+ lines away
            if let Some(next) = self.comments.front()
                && next.line >= comment.line + 2
                && next.line <= line
            {
                self.write_newline_without_fill();
            }

            self.last_ast_line = self.last_ast_line.max(comment.line);
            comments_written += 1;
        }

        // Remove trailing newline if we wrote any comments
        if comments_written > 0 && self.out.ends_with('\n') {
            self.out.pop();
        }
    }

    // --------------------------
    // Indent and scope functions
    // --------------------------

    /// Enter with a indent
    pub fn enter(&mut self) {
        self.indent += 1;
    }

    /// Leave with a dedent
    pub fn leave(&mut self) {
        self.indent -= 1;
    }

    pub(crate) fn push_tmp_buffer(&mut self) {
        self.tmp_buffers.push(String::new());
    }

    pub(crate) fn pop_tmp_buffer(&mut self) -> Option<String> {
        self.tmp_buffers.pop()
    }

    pub(crate) fn push_expr_span(&mut self, start_line: u64, end_line: u64) {
        self.expr_span_stack.push((start_line, end_line));
    }

    pub(crate) fn pop_expr_span(&mut self) {
        self.expr_span_stack.pop();
    }

    pub(crate) fn current_expr_end_line(&self) -> Option<u64> {
        self.expr_span_stack.last().map(|(_, end_line)| *end_line)
    }

    pub(crate) fn current_expr_start_line(&self) -> Option<u64> {
        self.expr_span_stack
            .last()
            .map(|(start_line, _)| *start_line)
    }

    /// Pop the next pending comment if it lives on `start_line` and emit it
    /// inline (with a leading space). Used by walkers that open a multi-line
    /// block (config `{`, list `[`) to preserve trailing comments that sit
    /// on the same line as the opening delimiter, e.g.
    ///
    /// ```text
    /// config = { # comment for the opening brace
    ///     key1: 1
    /// }
    /// ```
    ///
    /// Callers must ensure the next comment cannot belong to a child node
    /// on the same line (e.g. only call this when the first child entry is
    /// on a later line than `start_line`).
    pub(crate) fn pop_inline_trailing_comment_on_start_line(&mut self, start_line: u64) {
        if let Some(comment) = self.comments.front()
            && comment.line == start_line
        {
            let comment = self.comments.pop_front().unwrap();
            self.write_space();
            self.write(&comment.node.text);
        }
    }
}

/// Print AST to string with an explicit printer configuration. Use this when
/// the caller needs to override defaults (e.g. to honor `.editorconfig`).
/// The default format is according to the KCL code style defined here:
/// https://kcl-lang.io/docs/reference/lang/spec/codestyle
pub fn print_ast_module_with_config(module: &Module, cfg: Config) -> String {
    let mut printer = Printer::new(cfg, &NoHook);
    printer.write_module(module);
    // Trim trailing newlines to ensure exactly one newline at EOF
    let trimmed = printer.out.trim_end_matches('\n');
    if trimmed.is_empty() {
        trimmed.to_string()
    } else {
        format!("{}\n", trimmed)
    }
}

/// Print AST to string. The default format is according to the KCL code style defined here: https://kcl-lang.io/docs/reference/lang/spec/codestyle
pub fn print_ast_module(module: &Module) -> String {
    print_ast_module_with_config(module, Config::default())
}

/// Print AST to string
pub fn print_ast_node(node: ASTNode) -> String {
    let mut printer = Printer::default();
    printer.write_node(node);
    printer.out
}

/// Print schema expression AST node to string.
pub fn print_schema_expr(schema_expr: &ast::SchemaExpr) -> String {
    let mut printer = Printer::default();
    printer.walk_schema_expr(schema_expr);
    printer.out
}
