use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_ast::ast;

#[derive(Debug, Clone)]
enum Indentation {
    Indent = 0,
    Dedent = 1,
    Newline = 2,
    IndentWithNewline = 3,
    Fill = 5,
}

const INVALID_AST_MSG: &str = "Invalid AST Node";
const TEMP_ROOT: &str = "<root>";

const WHITESPACE: &str = " ";
const TAB: &str = "\t";
const NEWLINE: &str = "\n";
const 
