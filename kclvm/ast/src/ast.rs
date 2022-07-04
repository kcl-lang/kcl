//! """The `ast` file contains the definitions of all KCL AST nodes
//! and operators and all AST nodes are derived from the `AST` class.
//! The main structure of a KCL program is as follows:
//!
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                             Program                             │
//! │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
//! │  │   Main Package  │  │     Package1    │  │     Package2    │  │
//! │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │
//! │  │  │  Module1  │  │  │  │  Module1  │  │  │  │  Module1  │  │  │
//! │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │
//! │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │
//! │  │  │  Module2  │  │  │  │  Module2  │  │  │  │  Module2  │  │  │
//! │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │
//! │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │
//! │  │  │    ...    │  │  │  │    ...    │  │  │  │    ...    │  │  │
//! │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │  │
//! │  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘

//! A single KCL file represents a module, which records file information,
//! package path information, and module document information, which is
//! mainly composed of all the statements in the KCL file.

//! The combination of multiple KCL files is regarded as a complete KCL
//! Program. For example, a single KCL file can be imported into KCL
//! files in other packages through statements such as import. Therefore,
//! the Program is composed of multiple modules, and each module is
//! associated with it. Corresponding to the package path.

//! :note: When the definition of any AST node is modified or the AST node
//! is added/deleted, it is necessary to modify the corresponding processing
//! in the compiler and regenerate the walker code.
//! :copyright: Copyright 2020 The KCL Authors. All rights reserved.
//!
//! todo: remove type_str fields after python frontend removed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use kclvm_span::Loc;
use rustc_span::Pos;

use super::token;
use crate::node_ref;
/// Node is the file, line and column number information
/// that all AST nodes need to contain.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node<T> {
    pub node: T,
    pub filename: String,
    pub line: u64,
    pub column: u64,
    pub end_line: u64,
    pub end_column: u64,
}

impl<T> Node<T> {
    pub fn new(
        node: T,
        filename: String,
        line: u64,
        column: u64,
        end_line: u64,
        end_column: u64,
    ) -> Self {
        Self {
            node,
            filename,
            line,
            column,
            end_line,
            end_column,
        }
    }

    pub fn dummy_node(node: T) -> Self {
        Self {
            node,
            filename: "".to_string(),
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 1,
        }
    }

    pub fn node(node: T, (lo, hi): (Loc, Loc)) -> Self {
        Self {
            node,
            filename: format!("{}", lo.file.name.prefer_remapped()),
            line: lo.line as u64,
            column: lo.col.to_usize() as u64,
            end_line: hi.line as u64,
            end_column: hi.col.to_usize() as u64,
        }
    }

    pub fn node_with_pos(node: T, pos: (String, u64, u64, u64, u64)) -> Self {
        Self {
            node,
            filename: pos.0.clone(),
            line: pos.1,
            column: pos.2,
            end_line: pos.3,
            end_column: pos.4,
        }
    }

    pub fn pos(&self) -> (String, u64, u64, u64, u64) {
        (
            self.filename.clone(),
            self.line,
            self.column,
            self.end_line,
            self.end_column,
        )
    }

    pub fn set_pos(&mut self, pos: (String, u64, u64, u64, u64)) {
        self.filename = pos.0.clone();
        self.line = pos.1;
        self.column = pos.2;
        self.end_line = pos.3;
        self.end_column = pos.4;
    }
}

impl TryInto<Node<Identifier>> for Node<Expr> {
    type Error = &'static str;

    fn try_into(self) -> Result<Node<Identifier>, Self::Error> {
        match self.node {
            Expr::Identifier(ident) => Ok(Node {
                node: ident,
                filename: self.filename,
                line: self.line,
                column: self.column,
                end_line: self.end_line,
                end_column: self.end_column,
            }),
            _ => Err("invalid identifier"),
        }
    }
}

impl TryInto<Node<SchemaExpr>> for Node<Expr> {
    type Error = &'static str;

    fn try_into(self) -> Result<Node<SchemaExpr>, Self::Error> {
        match self.node {
            Expr::Schema(schema_expr) => Ok(Node {
                node: schema_expr,
                filename: self.filename,
                line: self.line,
                column: self.column,
                end_line: self.end_line,
                end_column: self.end_column,
            }),
            _ => Err("invalid schema expr"),
        }
    }
}

/// NodeRef<T> is the Box reference of Node<T> with the
/// AST node type T
pub type NodeRef<T> = Box<Node<T>>;

/// KCL command line argument spec, e.g. `kcl main.k -D name=value`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CmdArgSpec {
    pub name: String,
    pub value: String,
}

/// KCL command line override spec, e.g. `kcl main.k -O pkgpath:path.to.field=field_value`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OverrideSpec {
    pub pkgpath: String,
    pub field_path: String,
    pub field_value: String,
    pub action: OverrideAction,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OverrideAction {
    CreateOrUpdate,
    Delete,
}

/// Program is the AST collection of all files of the running KCL program.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Program {
    pub root: String,
    pub main: String,
    pub pkgs: HashMap<String, Vec<Module>>,
    pub cmd_args: Vec<CmdArgSpec>,
    pub cmd_overrides: Vec<OverrideSpec>,
}

impl Program {
    /// Get main entry files.
    pub fn get_main_files(&self) -> Vec<String> {
        match self.pkgs.get(crate::MAIN_PKG) {
            Some(modules) => modules.iter().map(|m| m.filename.clone()).collect(),
            None => vec![],
        }
    }
}

/// Module is an abstract syntax tree for a single KCL file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Module {
    pub filename: String,
    pub pkg: String,
    pub doc: String,
    pub name: String,
    pub body: Vec<NodeRef<Stmt>>,
    pub comments: Vec<NodeRef<Comment>>,
}

impl Module {
    /// Get all ast.schema_stmts from ast.module and return it in a Vec.
    pub fn filter_schema_stmt_from_module(&self) -> Vec<NodeRef<SchemaStmt>> {
        let mut stmts = Vec::new();
        for stmt in &self.body {
            if let Stmt::Schema(schema_stmt) = &stmt.node {
                stmts.push(node_ref!(schema_stmt.clone()));
            }
        }
        return stmts;
    }
}

/*
 * Stmt
 */

/// A statement
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Stmt {
    TypeAlias(TypeAliasStmt),
    Expr(ExprStmt),
    Unification(UnificationStmt),
    Assign(AssignStmt),
    AugAssign(AugAssignStmt),
    Assert(AssertStmt),
    If(IfStmt),
    Import(ImportStmt),
    SchemaAttr(SchemaAttr),
    Schema(SchemaStmt),
    Rule(RuleStmt),
}

/// TypeAliasStmt represents a type alias statement, e.g.
/// ```kcl
/// type StrOrInt = str | int
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypeAliasStmt {
    pub type_name: NodeRef<Identifier>,
    pub type_value: NodeRef<String>,

    #[serde(skip_serializing)]
    pub ty: Option<NodeRef<Type>>,
}

/// ExprStmt represents a expression statement, e.g.
/// ```kcl
/// 1
/// """A long string"""
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExprStmt {
    pub exprs: Vec<NodeRef<Expr>>,
}

/// UnificationStmt represents a declare statement with the union operator, e.g.
/// ```kcl
/// data: ASchema {}
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnificationStmt {
    pub target: NodeRef<Identifier>,
    pub value: NodeRef<SchemaExpr>,
}

/// AssignStmt represents an assignment, e.g.
/// ```kcl
/// a: int = 1
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssignStmt {
    pub targets: Vec<NodeRef<Identifier>>,
    pub value: NodeRef<Expr>,
    pub type_annotation: Option<NodeRef<String>>,

    #[serde(skip_serializing)]
    pub ty: Option<NodeRef<Type>>,
}

/// AugAssignStmt represents an argument assignment, e.g.
/// ```kcl
/// a += 1
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AugAssignStmt {
    pub target: NodeRef<Identifier>,
    pub value: NodeRef<Expr>,
    pub op: AugOp,
}

/// AssertStmt represents an assert statement, e.g.
/// ```kcl
/// assert True if condition, "Assert failed message"
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssertStmt {
    pub test: NodeRef<Expr>,
    pub if_cond: Option<NodeRef<Expr>>,
    pub msg: Option<NodeRef<Expr>>,
}

/// IfStmt, e.g.
/// ```kcl
/// if condition1:
///     if condition2:
///         a = 1
/// elif condition3:
///     b = 2
/// else:
///     c = 3
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IfStmt {
    pub body: Vec<NodeRef<Stmt>>,
    pub cond: NodeRef<Expr>,
    pub orelse: Vec<NodeRef<Stmt>>,
}

/// ImportStmt, e.g.
/// ```kcl
/// import pkg as pkg_alias
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImportStmt {
    pub path: String,
    pub rawpath: String,
    pub name: String,
    pub asname: Option<String>,
}

/// SchemaStmt, e.g.
/// ```kcl
/// schema BaseSchema:
///
/// schema SchemaExample(BaseSchema)[arg: str]:
///     """Schema documents"""
///     attr?: str = arg
///     check:
///         len(attr) > 3 if attr, "Check failed message"
///
/// mixin MixinExample for ProtocolExample:
///     attr: int
///
/// protocol ProtocolExample:
///     attr: int
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaStmt {
    pub doc: String,
    pub name: NodeRef<String>,
    pub parent_name: Option<NodeRef<Identifier>>,
    pub for_host_name: Option<NodeRef<Identifier>>,
    pub is_mixin: bool,
    pub is_protocol: bool,
    pub args: Option<NodeRef<Arguments>>,
    pub mixins: Vec<NodeRef<Identifier>>,
    pub body: Vec<NodeRef<Stmt>>,
    pub decorators: Vec<NodeRef<CallExpr>>,
    pub checks: Vec<NodeRef<CheckExpr>>,
    pub index_signature: Option<NodeRef<SchemaIndexSignature>>,
}

impl SchemaStmt {
    /// Get schema full attribute list (line, column, name) including
    /// un-exported attributes.
    pub fn get_left_identifier_list(&self) -> Vec<(u64, u64, String)> {
        let mut attr_list: Vec<(u64, u64, String)> = vec![];
        fn loop_body(body: &[NodeRef<Stmt>], attr_list: &mut Vec<(u64, u64, String)>) {
            for stmt in body {
                match &stmt.node {
                    Stmt::Unification(unification_stmt) => {
                        attr_list.push((
                            unification_stmt.target.line,
                            unification_stmt.target.column,
                            unification_stmt.target.node.names[0].to_string(),
                        ));
                    }
                    Stmt::Assign(assign_stmt) => {
                        for target in &assign_stmt.targets {
                            attr_list.push((
                                target.line,
                                target.column,
                                target.node.names[0].to_string(),
                            ));
                        }
                    }
                    Stmt::AugAssign(aug_assign_stmt) => {
                        attr_list.push((
                            aug_assign_stmt.target.line,
                            aug_assign_stmt.target.column,
                            aug_assign_stmt.target.node.names[0].to_string(),
                        ));
                    }
                    Stmt::If(if_stmt) => {
                        loop_body(&if_stmt.body, attr_list);
                        loop_body(&if_stmt.orelse, attr_list);
                    }
                    Stmt::SchemaAttr(schema_attr) => {
                        attr_list.push((
                            schema_attr.name.line,
                            schema_attr.name.column,
                            schema_attr.name.node.to_string(),
                        ));
                    }
                    _ => {}
                }
            }
        }
        loop_body(&self.body, &mut attr_list);
        attr_list
    }

    /// Whether the schema contains only attribute definitions.
    pub fn has_only_attribute_definitions(&self) -> bool {
        self.args.is_none()
            && self.mixins.is_empty()
            && self.checks.is_empty()
            && self
                .body
                .iter()
                .all(|stmt| matches!(stmt.node, Stmt::SchemaAttr(_)))
    }
}

/// SchemaIndexSignature, e.g.
/// ```kcl
/// schema SchemaIndexSignatureExample:
///     [str]: int
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaIndexSignature {
    pub key_name: Option<String>,
    pub key_type: NodeRef<String>,
    pub value_type: NodeRef<String>,
    pub value: Option<NodeRef<Expr>>,
    pub any_other: bool,

    #[serde(skip_serializing)]
    pub value_ty: Option<NodeRef<Type>>,
}

/// SchemaAttr, e.g.
/// ```kcl
/// schema SchemaAttrExample:
///      x: int
///      y: str
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaAttr {
    pub doc: String,
    pub name: NodeRef<String>,
    pub type_str: NodeRef<String>,
    pub op: Option<BinOrAugOp>,
    pub value: Option<NodeRef<Expr>>,
    pub is_optional: bool,
    pub decorators: Vec<NodeRef<CallExpr>>,

    #[serde(skip_serializing)]
    pub ty: Option<NodeRef<Type>>,
}

/// RuleStmt, e.g.
/// ```kcl
/// rule RuleExample:
///     a > 1
///     b < 0
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuleStmt {
    pub doc: String,
    pub name: NodeRef<String>,
    pub parent_rules: Vec<NodeRef<Identifier>>,
    pub decorators: Vec<NodeRef<CallExpr>>,
    pub checks: Vec<NodeRef<CheckExpr>>,
    pub args: Option<NodeRef<Arguments>>,
    pub for_host_name: Option<NodeRef<Identifier>>,
}

/// A expression
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
    Identifier(Identifier),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    If(IfExpr),
    Selector(SelectorExpr),
    Call(CallExpr),
    Paren(ParenExpr),
    Quant(QuantExpr),
    List(ListExpr),
    ListIfItem(ListIfItemExpr),
    ListComp(ListComp),
    Starred(StarredExpr),
    DictComp(DictComp),
    ConfigIfEntry(ConfigIfEntryExpr),
    CompClause(CompClause),
    Schema(SchemaExpr),
    Config(ConfigExpr),
    Check(CheckExpr),
    Lambda(LambdaExpr),
    Subscript(Subscript),
    Keyword(Keyword),
    Arguments(Arguments),
    Compare(Compare),
    NumberLit(NumberLit),
    StringLit(StringLit),
    NameConstantLit(NameConstantLit),
    JoinedString(JoinedString),
    FormattedValue(FormattedValue),
}

/// Identifier, e.g.
/// ```kcl
/// a
/// b
/// _c
/// pkg.a
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identifier {
    pub names: Vec<String>,
    pub pkgpath: String,
    pub ctx: ExprContext,
}

impl Identifier {
    pub fn get_name(&self) -> String {
        self.names.join(".")
    }
}

/// UnaryExpr, e.g.
/// ```kcl
/// +1
/// -2
/// ~3
/// not True
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: NodeRef<Expr>,
}

/// BinaryExpr, e.g.
/// ```kcl
/// 1 + 1
/// 3 - 2
/// 5 / 2
/// a is None
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BinaryExpr {
    pub left: NodeRef<Expr>,
    pub op: BinOrCmpOp,
    pub right: NodeRef<Expr>,
}

/// IfExpr, e.g.
/// ```kcl
/// 1 if condition else 2
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IfExpr {
    pub body: NodeRef<Expr>,
    pub cond: NodeRef<Expr>,
    pub orelse: NodeRef<Expr>,
}

/// SelectorExpr, e.g.
/// ```kcl
/// x.y
/// x?.y
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SelectorExpr {
    pub value: NodeRef<Expr>,
    pub attr: NodeRef<Identifier>,
    pub ctx: ExprContext,
    pub has_question: bool,
}

/// CallExpr, e.g.
/// ```kcl
/// func1()
/// func2(1)
/// func3(x=2)
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallExpr {
    pub func: NodeRef<Expr>,
    pub args: Vec<NodeRef<Expr>>,
    pub keywords: Vec<NodeRef<Keyword>>,
}

/// ParenExpr, e.g.
/// ```kcl
/// 1 + (2 - 3)
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParenExpr {
    pub expr: NodeRef<Expr>,
}

/// QuantExpr, e.g.
/// ```kcl
/// all x in collection {x > 0}
/// any y in collection {y < 0}
/// map x in collection {x + 1}
/// filter x in collection {x > 1}
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuantExpr {
    pub target: NodeRef<Expr>,
    pub variables: Vec<NodeRef<Identifier>>,
    pub op: QuantOperation,
    pub test: NodeRef<Expr>,
    pub if_cond: Option<NodeRef<Expr>>,
    pub ctx: ExprContext,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QuantOperation {
    All,
    Any,
    Filter,
    Map,
}

impl Into<String> for QuantOperation {
    fn into(self) -> String {
        let s = match self {
            QuantOperation::All => "all",
            QuantOperation::Any => "any",
            QuantOperation::Filter => "filter",
            QuantOperation::Map => "map",
        };

        s.to_string()
    }
}

/// ListExpr, e.g.
/// ```kcl
/// [1, 2, 3]
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListExpr {
    pub elts: Vec<NodeRef<Expr>>,
    pub ctx: ExprContext,
}

/// ListIfItemExpr, e.g.
/// ```kcl
/// [1, if condition: 2, 3]
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListIfItemExpr {
    pub if_cond: NodeRef<Expr>,
    pub exprs: Vec<NodeRef<Expr>>,
    pub orelse: Option<NodeRef<Expr>>,
}

pub enum CompType {
    List,
    Dict,
}

/// ListComp, e.g.
/// ```kcl
/// [x ** 2 for x in [1, 2, 3]]
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListComp {
    pub elt: NodeRef<Expr>,
    pub generators: Vec<NodeRef<CompClause>>,
}

/// StarredExpr, e.g.
/// ```kcl
/// [1, 2, *[3, 4]]
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StarredExpr {
    pub value: NodeRef<Expr>,
    pub ctx: ExprContext,
}

/// DictComp, e.g.
/// ```kcl
/// {k: v + 1 for k, v in {k1 = 1, k2 = 2}}
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DictComp {
    pub entry: ConfigEntry,
    pub generators: Vec<NodeRef<CompClause>>,
}

/// ConfigIfEntryExpr, e.g.
/// ```kcl
/// {
///     k1 = 1
///     if condition:
///         k2 = 2
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigIfEntryExpr {
    pub if_cond: NodeRef<Expr>,
    pub items: Vec<NodeRef<ConfigEntry>>,
    pub orelse: Option<NodeRef<Expr>>,
}

/// CompClause, e.g.
/// ```kcl
/// i, a in [1, 2, 3] if i > 1 and a > 1
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompClause {
    pub targets: Vec<NodeRef<Identifier>>,
    pub iter: NodeRef<Expr>,
    pub ifs: Vec<NodeRef<Expr>>,
}

/// SchemaExpr, e.g.
/// ```kcl
/// ASchema(arguments) {
///     attr1 = 1
///     attr2 = 2
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaExpr {
    pub name: NodeRef<Identifier>,
    pub args: Vec<NodeRef<Expr>>,
    pub kwargs: Vec<NodeRef<Keyword>>,
    pub config: NodeRef<Expr>,
}

/// ConfigExpr, e.g.
/// ```kcl
/// {
///     attr1 = 1
///     attr2 = 2
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigExpr {
    pub items: Vec<NodeRef<ConfigEntry>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigEntryOperation {
    Union,
    Override,
    Insert,
}

impl ConfigEntryOperation {
    pub fn value(&self) -> i32 {
        match self {
            ConfigEntryOperation::Union => 0,
            ConfigEntryOperation::Override => 1,
            ConfigEntryOperation::Insert => 2,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            ConfigEntryOperation::Union => ":",
            ConfigEntryOperation::Override => "=",
            ConfigEntryOperation::Insert => "+=",
        }
    }
}

/// ConfigEntry, e.g.
/// ```kcl
/// {
///     a = 1
///     b: 1
///     c += [0]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigEntry {
    pub key: Option<NodeRef<Expr>>,
    pub value: NodeRef<Expr>,
    pub operation: ConfigEntryOperation,
    pub insert_index: isize,
}

/// CheckExpr, e.g.
/// ```kcl
/// len(attr) > 3 if attr, "Check failed message"
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckExpr {
    pub test: NodeRef<Expr>,
    pub if_cond: Option<NodeRef<Expr>>,
    pub msg: Option<NodeRef<Expr>>,
}

/// LambdaExpr, e.g.
/// ```kcl
/// lambda x, y {
///     z = 2 * x
///     z + y
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LambdaExpr {
    pub args: Option<NodeRef<Arguments>>,
    pub return_type_str: Option<String>,
    pub body: Vec<NodeRef<Stmt>>,

    #[serde(skip_serializing)]
    pub return_ty: Option<NodeRef<Type>>,
}

/// Subscript, e.g.
/// ```kcl
/// a[0]
/// b["k"]
/// c?[1]
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscript {
    pub value: NodeRef<Expr>,
    pub index: Option<NodeRef<Expr>>,
    pub lower: Option<NodeRef<Expr>>,
    pub upper: Option<NodeRef<Expr>>,
    pub step: Option<NodeRef<Expr>>,
    pub ctx: ExprContext,
    pub has_question: bool,
}

/// Keyword, e.g.
/// ```kcl
/// arg=value
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keyword {
    pub arg: NodeRef<Identifier>,
    pub value: Option<NodeRef<Expr>>,
}

/// Arguments, e.g.
/// ```kcl
/// lambda x: int = 1, y: int = 1 {
///     x + y
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arguments {
    pub args: Vec<NodeRef<Identifier>>,
    pub defaults: Vec<Option<NodeRef<Expr>>>,
    pub type_annotation_list: Vec<Option<NodeRef<String>>>,

    #[serde(skip_serializing)]
    #[serde(default)]
    pub ty_list: Vec<Option<NodeRef<Type>>>,
}

impl Arguments {
    pub fn get_arg_type(&self, i: usize) -> Type {
        self.ty_list[i]
            .as_ref()
            .map_or(Type::Any, |ty| ty.node.clone())
    }
}

/// Compare, e.g.
/// ```kcl
/// 0 < a < 10
/// b is not None
/// c != d
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Compare {
    pub left: NodeRef<Expr>,
    pub ops: Vec<CmpOp>,
    pub comparators: Vec<NodeRef<Expr>>,
}

/// Literal, e.g.
/// ```kcl
/// 1
/// 2.0
/// "string literal"
/// """long string literal"""
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Literal {
    Number(NumberLit),
    String(StringLit),
    NameConstant(NameConstantLit),
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NumberBinarySuffix {
    n,
    u,
    m,
    k,
    K,
    M,
    G,
    T,
    P,
    Ki,
    Mi,
    Gi,
    Ti,
    Pi,
}

impl TryFrom<&str> for NumberBinarySuffix {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "n" => Ok(NumberBinarySuffix::n),
            "u" => Ok(NumberBinarySuffix::u),
            "m" => Ok(NumberBinarySuffix::m),
            "k" => Ok(NumberBinarySuffix::k),
            "K" => Ok(NumberBinarySuffix::K),
            "M" => Ok(NumberBinarySuffix::M),
            "G" => Ok(NumberBinarySuffix::G),
            "T" => Ok(NumberBinarySuffix::T),
            "P" => Ok(NumberBinarySuffix::P),
            "Ki" => Ok(NumberBinarySuffix::Ki),
            "Mi" => Ok(NumberBinarySuffix::Mi),
            "Gi" => Ok(NumberBinarySuffix::Gi),
            "Ti" => Ok(NumberBinarySuffix::Ti),
            "Pi" => Ok(NumberBinarySuffix::Pi),
            _ => Err("invalid number binary suffix"),
        }
    }
}

impl NumberBinarySuffix {
    pub fn value(&self) -> String {
        format!("{:?}", self)
    }
    /// Get all names of NumberBinarySuffix
    #[inline]
    pub const fn all_names() -> &'static [&'static str] {
        &[
            "n", "u", "m", "k", "K", "M", "G", "T", "P", "Ki", "Mi", "Gi", "Ti", "Pi", "i",
        ]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NumberLitValue {
    Int(i64),
    Float(f64),
}

/// NumberLit, e.g.
/// ```kcl
/// 1m
/// 1K
/// 1Mi
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberLit {
    pub binary_suffix: Option<NumberBinarySuffix>,
    pub value: NumberLitValue,
}

/// StringLit, e.g.
/// ```kcl
/// "string literal"
/// """long string literal"""
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StringLit {
    pub is_long_string: bool,
    pub raw_value: String,
    pub value: String,
}

/// Generate ast.StringLit from String
impl TryFrom<String> for StringLit {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.clone(),
            raw_value: format!("{:?}", value),
            is_long_string: false,
        })
    }
}

/// NameConstant, e.g.
/// ```kcl
/// True
/// False
/// None
/// Undefined
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NameConstant {
    True,
    False,
    None,
    Undefined,
}

impl NameConstant {
    pub fn symbol(&self) -> &'static str {
        match self {
            NameConstant::True => "True",
            NameConstant::False => "False",
            NameConstant::None => "None",
            NameConstant::Undefined => "Undefined",
        }
    }
}

/// Generate ast.NameConstant from Bool
impl TryFrom<bool> for NameConstant {
    type Error = &'static str;

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        match value {
            true => Ok(NameConstant::True),
            false => Ok(NameConstant::False),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameConstantLit {
    pub value: NameConstant,
}

/// JoinedString, e.g. abc in the string interpolation "${var1} abc ${var2}"
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinedString {
    pub is_long_string: bool,
    pub values: Vec<NodeRef<Expr>>,
    pub raw_value: String,
}

/// FormattedValue, e.g. var1 and var2  in the string interpolation "${var1} abc ${var2}"
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormattedValue {
    pub is_long_string: bool,
    pub value: NodeRef<Expr>,
    pub format_spec: Option<String>,
}

/// Comment, e.g.
/// ```kcl
/// # This is a comment
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    pub text: String,
}

/*
 * Operators and context
 */

/// BinOp is the set of all binary operators in KCL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinOp {
    /// The `+` operator (addition)
    Add,
    /// The `-` operator (subtraction)
    Sub,
    /// The `*` operator (multiplication)
    Mul,
    /// The `/` operator (division)
    Div,
    /// The `%` operator (modulus)
    Mod,
    /// The `**` operator (power)
    Pow,
    /// The `//` operator (floor division)
    FloorDiv,
    /// The `<<` operator (shift left)
    LShift,
    /// The `>>` operator (shift right)
    RShift,
    /// The `^` operator (bitwise xor)
    BitXor,
    /// The `&` operator (bitwise and)
    BitAnd,
    /// The `|` operator (bitwise or)
    BitOr,
    /// The `and` operator (logical and)
    And,
    /// The `or` operator (logical or)
    Or,
    /// The `as` operator (type cast)
    As,
}

impl BinOp {
    pub fn all_symbols() -> Vec<String> {
        vec![
            BinOp::Add.symbol().to_string(),
            BinOp::Sub.symbol().to_string(),
            BinOp::Mul.symbol().to_string(),
            BinOp::Div.symbol().to_string(),
            BinOp::Mod.symbol().to_string(),
            BinOp::Pow.symbol().to_string(),
            BinOp::FloorDiv.symbol().to_string(),
            BinOp::LShift.symbol().to_string(),
            BinOp::RShift.symbol().to_string(),
            BinOp::BitXor.symbol().to_string(),
            BinOp::BitAnd.symbol().to_string(),
            BinOp::BitOr.symbol().to_string(),
            BinOp::And.symbol().to_string(),
            BinOp::Or.symbol().to_string(),
            BinOp::As.symbol().to_string(),
        ]
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
            BinOp::FloorDiv => "//",
            BinOp::LShift => "<<",
            BinOp::RShift => ">>",
            BinOp::BitXor => "^",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::As => "as",
        }
    }
}

/// BinOp is the set of all argument operators in KCL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AugOp {
    // The `=` operator (assign)
    Assign,
    /// The `+=` operator (addition)
    Add,
    /// The `-=` operator (subtraction)
    Sub,
    /// The `*=` operator (multiplication)
    Mul,
    /// The `/=` operator (division)
    Div,
    /// The `%=` operator (modulus)
    Mod,
    /// The `**=` operator (power)
    Pow,
    /// The `//=` operator (floor division)
    FloorDiv,
    /// The `<<=` operator (shift left)
    LShift,
    /// The `>>=` operator (shift right)
    RShift,
    /// The `^=` operator (bitwise xor)
    BitXor,
    /// The `&=` operator (bitwise and)
    BitAnd,
    /// The `|=` operator (bitwise or)
    BitOr,
}

impl AugOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            AugOp::Assign => "=",
            AugOp::Add => "+=",
            AugOp::Sub => "-=",
            AugOp::Mul => "*=",
            AugOp::Div => "/=",
            AugOp::Mod => "%=",
            AugOp::Pow => "**=",
            AugOp::FloorDiv => "//=",
            AugOp::LShift => "<<=",
            AugOp::RShift => ">>=",
            AugOp::BitXor => "^=",
            AugOp::BitAnd => "&=",
            AugOp::BitOr => "|=",
        }
    }
}

impl TryInto<BinOp> for AugOp {
    type Error = &'static str;

    fn try_into(self) -> Result<BinOp, Self::Error> {
        match self {
            AugOp::Add => Ok(BinOp::Add),
            AugOp::Sub => Ok(BinOp::Sub),
            AugOp::Mul => Ok(BinOp::Mul),
            AugOp::Div => Ok(BinOp::Div),
            AugOp::Mod => Ok(BinOp::Mod),
            AugOp::Pow => Ok(BinOp::Pow),
            AugOp::FloorDiv => Ok(BinOp::FloorDiv),
            AugOp::LShift => Ok(BinOp::LShift),
            AugOp::RShift => Ok(BinOp::RShift),
            AugOp::BitXor => Ok(BinOp::BitXor),
            AugOp::BitAnd => Ok(BinOp::And),
            AugOp::BitOr => Ok(BinOp::BitOr),
            _ => Err("aug assign op can not into bin op"),
        }
    }
}

/// UnaryOp is the set of all unary operators in KCL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UnaryOp {
    /// The `+` operator for positive
    UAdd,
    /// The `-` operator for negation
    USub,
    /// The `~` operator for bitwise negation
    Invert,
    /// The `not` operator for logical inversion
    Not,
}

impl UnaryOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            UnaryOp::UAdd => "+",
            UnaryOp::USub => "-",
            UnaryOp::Invert => "~",
            UnaryOp::Not => "not",
        }
    }
}

/// CmpOp is the set of all comparison operators in KCL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CmpOp {
    /// The `==` operator (equality)
    Eq,
    /// The `!=` operator (not equal to)
    NotEq,
    /// The `<` operator (less than)
    Lt,
    /// The `<=` operator (less than or equal to)
    LtE,
    /// The `>` operator (greater than)
    Gt,
    /// The `>=` operator (greater than or equal to)
    GtE,
    /// The `is` operator (greater than or equal to)
    Is,
    /// The `in` operator
    In,
    /// The `not in` operator
    NotIn,
    /// The `not` operator
    Not,
    /// The `is not` operator
    IsNot,
}

impl CmpOp {
    pub fn all_symbols() -> Vec<String> {
        vec![
            CmpOp::Eq.symbol().to_string(),
            CmpOp::NotEq.symbol().to_string(),
            CmpOp::Lt.symbol().to_string(),
            CmpOp::LtE.symbol().to_string(),
            CmpOp::Gt.symbol().to_string(),
            CmpOp::GtE.symbol().to_string(),
            CmpOp::Is.symbol().to_string(),
            CmpOp::In.symbol().to_string(),
            CmpOp::NotIn.symbol().to_string(),
            CmpOp::Not.symbol().to_string(),
            CmpOp::IsNot.symbol().to_string(),
        ]
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            CmpOp::Eq => "==",
            CmpOp::NotEq => "!=",
            CmpOp::Lt => "<",
            CmpOp::LtE => "<=",
            CmpOp::Gt => ">",
            CmpOp::GtE => ">=",
            CmpOp::Is => "is",
            CmpOp::In => "in",
            CmpOp::NotIn => "not in",
            CmpOp::Not => "not",
            CmpOp::IsNot => "is not",
        }
    }
}

/// BinOrCmpOp is the set of all binary and comparison operators in KCL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinOrCmpOp {
    Bin(BinOp),
    Cmp(CmpOp),
}

/// BinOrAugOp is the set of all binary and argument operators in KCL.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinOrAugOp {
    Bin(BinOp),
    Aug(AugOp),
}

/// ExprContext represents the location information of the AST node.
/// The left side of the assignment symbol represents `Store`,
/// and the right side represents `Load`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExprContext {
    Load,
    Store,
}

/// A expression
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    Any,
    Named(Identifier),
    Basic(BasicType),
    List(ListType),
    Dict(DictType),
    Union(UnionType),
    Literal(LiteralType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BasicType {
    Bool,
    Int,
    Float,
    Str,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListType {
    pub inner_type: Option<NodeRef<Type>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DictType {
    pub key_type: Option<NodeRef<Type>>,
    pub value_type: Option<NodeRef<Type>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnionType {
    pub type_elements: Vec<NodeRef<Type>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LiteralType {
    Bool(bool),
    Int(i64, Option<NumberBinarySuffix>), // value + suffix
    Float(f64),
    Str(String),
}

impl ToString for Type {
    fn to_string(&self) -> String {
        fn to_str(typ: &Type, w: &mut String) {
            match typ {
                Type::Any => w.push_str("any"),
                Type::Named(x) => {
                    w.push_str(&x.names.join("."));
                }
                Type::Basic(x) => {
                    w.push_str(match x {
                        BasicType::Bool => "bool",
                        BasicType::Int => "int",
                        BasicType::Float => "float",
                        BasicType::Str => "str",
                    });
                }
                Type::List(x) => {
                    w.push('[');
                    if let Some(t) = &x.inner_type {
                        to_str(&t.node, w);
                    }
                    w.push(']');
                }
                Type::Dict(x) => {
                    w.push('{');
                    if let Some(t) = &x.key_type {
                        to_str(&t.node, w);
                    }
                    w.push(':');
                    if let Some(t) = &x.value_type {
                        to_str(&t.node, w);
                    }
                    w.push('}');
                }
                Type::Union(x) => {
                    if x.type_elements.is_empty() {
                        w.push_str("any");
                        return;
                    }
                    for (i, t) in x.type_elements.iter().enumerate() {
                        if i > 0 {
                            w.push('|');
                        }
                        to_str(&t.node, w);
                    }
                }

                Type::Literal(x) => match x {
                    LiteralType::Bool(v) => {
                        if *v {
                            w.push_str("True");
                        } else {
                            w.push_str("False");
                        }
                    }
                    LiteralType::Int(v, suffix) => {
                        if let Some(suffix) = suffix {
                            w.push_str(&format!("{}{}", v, suffix.value()));
                        } else {
                            w.push_str(&v.to_string());
                        }
                    }
                    LiteralType::Float(v) => {
                        let mut float_str = v.to_string();
                        if !float_str.contains('.') {
                            float_str.push_str(".0");
                        }
                        w.push_str(&float_str);
                    }
                    LiteralType::Str(v) => {
                        w.push_str(&format!("\"{}\"", v.replace('"', "\\\"")));
                    }
                },
            }
        }

        let mut result = "".to_string();
        to_str(self, &mut result);
        result
    }
}

impl From<token::BinOpToken> for AugOp {
    fn from(op_kind: token::BinOpToken) -> Self {
        match op_kind {
            token::BinOpToken::Plus => AugOp::Add,
            token::BinOpToken::Minus => AugOp::Sub,
            token::BinOpToken::Star => AugOp::Mul,
            token::BinOpToken::Slash => AugOp::Div,
            token::BinOpToken::Percent => AugOp::Mod,
            token::BinOpToken::StarStar => AugOp::Pow,
            token::BinOpToken::SlashSlash => AugOp::Add,
            token::BinOpToken::Caret => AugOp::BitXor,
            token::BinOpToken::And => AugOp::BitAnd,
            token::BinOpToken::Or => AugOp::BitOr,
            token::BinOpToken::Shl => AugOp::LShift,
            token::BinOpToken::Shr => AugOp::RShift,
        }
    }
}

impl TryFrom<token::Token> for UnaryOp {
    type Error = ();

    fn try_from(token: token::Token) -> Result<Self, Self::Error> {
        use kclvm_span::symbol::kw;

        match token.kind {
            token::TokenKind::UnaryOp(token::UnaryOpToken::UTilde) => Ok(UnaryOp::Invert),
            token::TokenKind::UnaryOp(token::UnaryOpToken::UNot) => Ok(UnaryOp::Not),
            token::TokenKind::BinOp(token::BinOpToken::Plus) => Ok(UnaryOp::UAdd),
            token::TokenKind::BinOp(token::BinOpToken::Minus) => Ok(UnaryOp::USub),
            _ => {
                if token.is_keyword(kw::Not) {
                    Ok(UnaryOp::Not)
                } else {
                    Err(())
                }
            }
        }
    }
}

impl BinOrCmpOp {
    pub fn all_symbols() -> Vec<String> {
        let mut result = vec![];
        result.append(&mut BinOp::all_symbols());
        result.append(&mut CmpOp::all_symbols());
        result
    }
}

impl TryFrom<token::Token> for BinOrCmpOp {
    type Error = ();

    fn try_from(token: token::Token) -> Result<Self, Self::Error> {
        use kclvm_span::symbol::kw;

        match token.kind {
            token::TokenKind::BinOp(ot) => match ot {
                token::BinOpToken::Plus => Ok(BinOrCmpOp::Bin(BinOp::Add)),
                token::BinOpToken::Minus => Ok(BinOrCmpOp::Bin(BinOp::Sub)),
                token::BinOpToken::Star => Ok(BinOrCmpOp::Bin(BinOp::Mul)),
                token::BinOpToken::Slash => Ok(BinOrCmpOp::Bin(BinOp::Div)),
                token::BinOpToken::Percent => Ok(BinOrCmpOp::Bin(BinOp::Mod)),
                token::BinOpToken::StarStar => Ok(BinOrCmpOp::Bin(BinOp::Pow)),
                token::BinOpToken::SlashSlash => Ok(BinOrCmpOp::Bin(BinOp::FloorDiv)),
                token::BinOpToken::Caret => Ok(BinOrCmpOp::Bin(BinOp::BitXor)),
                token::BinOpToken::And => Ok(BinOrCmpOp::Bin(BinOp::BitAnd)),
                token::BinOpToken::Or => Ok(BinOrCmpOp::Bin(BinOp::BitOr)),
                token::BinOpToken::Shl => Ok(BinOrCmpOp::Bin(BinOp::LShift)),
                token::BinOpToken::Shr => Ok(BinOrCmpOp::Bin(BinOp::RShift)),
            },
            token::TokenKind::BinCmp(ct) => match ct {
                token::BinCmpToken::Eq => Ok(BinOrCmpOp::Cmp(CmpOp::Eq)),
                token::BinCmpToken::NotEq => Ok(BinOrCmpOp::Cmp(CmpOp::NotEq)),
                token::BinCmpToken::Lt => Ok(BinOrCmpOp::Cmp(CmpOp::Lt)),
                token::BinCmpToken::LtEq => Ok(BinOrCmpOp::Cmp(CmpOp::LtE)),
                token::BinCmpToken::Gt => Ok(BinOrCmpOp::Cmp(CmpOp::Gt)),
                token::BinCmpToken::GtEq => Ok(BinOrCmpOp::Cmp(CmpOp::GtE)),
            },
            _ => {
                if token.is_keyword(kw::As) {
                    Ok(BinOrCmpOp::Bin(BinOp::As))
                } else if token.is_keyword(kw::Or) {
                    Ok(BinOrCmpOp::Bin(BinOp::Or))
                } else if token.is_keyword(kw::And) {
                    Ok(BinOrCmpOp::Bin(BinOp::And))
                } else if token.is_keyword(kw::In) {
                    Ok(BinOrCmpOp::Cmp(CmpOp::In))
                } else if token.is_keyword(kw::Is) {
                    Ok(BinOrCmpOp::Cmp(CmpOp::Is))
                } else {
                    Err(())
                }
            }
        }
    }
}
