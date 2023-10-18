use indexmap::IndexMap;
use kclvm_ast::{
    ast::{self, Module},
    token::TokenKind,
    walker::MutSelfTypedResultWalker,
};
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
#[derive(Debug)]
pub struct Config {
    pub tab_len: usize,
    pub indent_len: usize,
    pub use_spaces: bool,
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
    pub indent: usize,
    pub cfg: Config,
    /// Print comments,
    pub last_ast_line: u64,
    pub comments: VecDeque<ast::NodeRef<ast::Comment>>,
    pub import_spec: IndexMap<String, String>,
    pub hook: &'p (dyn PrinterHook + 'p),
}

impl Default for Printer<'_> {
    fn default() -> Self {
        Self {
            out: Default::default(),
            indent: Default::default(),
            cfg: Default::default(),
            last_ast_line: Default::default(),
            comments: Default::default(),
            import_spec: Default::default(),
            hook: &NoHook,
        }
    }
}

impl<'p> Printer<'p> {
    pub fn new(cfg: Config, hook: &'p (dyn PrinterHook + 'p)) -> Self {
        Self {
            out: "".to_string(),
            indent: 0,
            cfg,
            last_ast_line: 0,
            comments: VecDeque::default(),
            import_spec: IndexMap::default(),
            hook,
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
        self.out.push_str(string);
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
    pub fn write_ast_comments<T>(&mut self, node: &ast::NodeRef<T>) {
        if !self.cfg.write_comments {
            return;
        }
        if node.line > self.last_ast_line {
            self.last_ast_line = node.line;
            let mut index = None;
            for (i, comment) in self.comments.iter().enumerate() {
                if comment.line <= node.line {
                    index = Some(i);
                } else {
                    break;
                }
            }
            if let Some(index) = index {
                let mut count = index as isize;
                while count >= 0 {
                    match self.comments.pop_front() {
                        Some(comment) => {
                            self.writeln(&comment.node.text);
                        }
                        None => break,
                    }
                    count -= 1;
                }
            }
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
}

/// Print AST to string. The default format is according to the KCL code style defined here: https://kcl-lang.io/docs/reference/lang/spec/codestyle
pub fn print_ast_module(module: &Module) -> String {
    let mut printer = Printer::default();
    printer.write_module(module);
    printer.out
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
