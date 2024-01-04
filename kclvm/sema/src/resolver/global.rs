use std::cell::RefCell;
use std::sync::Arc;

use crate::info::is_private_field;
use crate::resolver::Resolver;
use crate::ty::{
    is_upper_bound, DecoratorTarget, FunctionType, Parameter, SchemaAttr, SchemaIndexSignature,
    SchemaType, Type, TypeKind, RESERVED_TYPE_IDENTIFIERS,
};
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast_pretty::{print_ast_node, print_schema_expr, ASTNode};
use kclvm_error::*;

use super::doc::parse_doc_string;
use super::scope::{ScopeObject, ScopeObjectKind};
use kclvm_ast::pos::GetPos;

const MAX_SCOPE_SCAN_COUNT: usize = 3;
pub const MIXIN_SUFFIX: &str = "Mixin";
pub const PROTOCOL_SUFFIX: &str = "Protocol";

impl<'ctx> Resolver<'ctx> {
    /// Init global types including top-level global variable types and
    /// schema types. Because the schema allows backward references,
    /// we scan multiple times.
    pub(crate) fn init_global_types(&mut self) {
        // 1. Scan all schema and rule type symbols
        let pkgpath = &self.ctx.pkgpath;
        match self.program.pkgs.get(pkgpath) {
            Some(modules) => {
                // 1. Scan all schema and rule type symbol
                for module in modules {
                    let pkgpath = &self.ctx.pkgpath.clone();
                    let filename = &module.filename;
                    self.change_package_context(pkgpath, filename);
                    for stmt in &module.body {
                        let (start, end) = stmt.get_span_pos();
                        let (name, doc, is_mixin, is_protocol, is_rule) = match &stmt.node {
                            ast::Stmt::Schema(schema_stmt) => (
                                &schema_stmt.name.node,
                                {
                                    if let Some(doc) = &schema_stmt.doc {
                                        doc.node.clone()
                                    } else {
                                        "".to_string()
                                    }
                                },
                                schema_stmt.is_mixin,
                                schema_stmt.is_protocol,
                                false,
                            ),
                            ast::Stmt::Rule(rule_stmt) => (
                                &rule_stmt.name.node,
                                {
                                    if let Some(doc) = &rule_stmt.doc {
                                        doc.node.clone()
                                    } else {
                                        "".to_string()
                                    }
                                },
                                false,
                                false,
                                true,
                            ),

                            _ => continue,
                        };
                        if self.contains_object(name) {
                            self.handler.add_error(
                                ErrorKind::UniqueKeyError,
                                &[Message {
                                    range: stmt.get_span_pos(),
                                    style: Style::LineAndColumn,
                                    message: format!("unique key error name '{}'", name),
                                    note: None,
                                    suggested_replacement: None,
                                }],
                            );
                            continue;
                        }
                        let parsed_doc = parse_doc_string(&doc);
                        let schema_ty = SchemaType {
                            name: name.to_string(),
                            pkgpath: self.ctx.pkgpath.clone(),
                            filename: self.ctx.filename.clone(),
                            doc: parsed_doc.summary.clone(),
                            examples: parsed_doc.examples,
                            is_instance: false,
                            is_mixin,
                            is_protocol,
                            is_rule,
                            base: None,
                            protocol: None,
                            mixins: vec![],
                            attrs: IndexMap::default(),
                            func: Box::new(FunctionType {
                                doc: parsed_doc.summary.clone(),
                                params: vec![],
                                self_ty: None,
                                return_ty: Arc::new(Type::VOID),
                                is_variadic: false,
                                kw_only_index: None,
                            }),
                            index_signature: None,
                            decorators: vec![],
                        };
                        self.insert_object(
                            name,
                            ScopeObject {
                                name: name.to_string(),
                                start,
                                end,
                                ty: Arc::new(Type::schema(schema_ty.clone())),
                                kind: ScopeObjectKind::Definition,
                                doc: Some(parsed_doc.summary.clone()),
                            },
                        )
                    }
                }
                // 2. Scan all variable type symbol
                self.init_global_var_types(true);
                // 3. Build all schema types
                for i in 0..MAX_SCOPE_SCAN_COUNT {
                    for module in modules {
                        let pkgpath = &self.ctx.pkgpath.clone();
                        let filename = &module.filename;
                        self.change_package_context(pkgpath, filename);
                        for stmt in &module.body {
                            let (start, end) = stmt.get_span_pos();
                            let schema_ty = match &stmt.node {
                                ast::Stmt::Schema(schema_stmt) => {
                                    let parent_ty = self.build_schema_parent_type(schema_stmt);
                                    let protocol_ty = self.build_schema_protocol_type(schema_stmt);
                                    self.build_schema_type(
                                        schema_stmt,
                                        parent_ty,
                                        protocol_ty,
                                        i == MAX_SCOPE_SCAN_COUNT - 1,
                                    )
                                }
                                ast::Stmt::Rule(rule_stmt) => {
                                    let protocol_ty = self.build_rule_protocol_type(rule_stmt);
                                    self.build_rule_type(
                                        rule_stmt,
                                        protocol_ty,
                                        i == MAX_SCOPE_SCAN_COUNT - 1,
                                    )
                                }
                                _ => continue,
                            };
                            self.insert_object(
                                &schema_ty.name.clone(),
                                ScopeObject {
                                    name: schema_ty.name.to_string(),
                                    start,
                                    end,
                                    ty: Arc::new(Type::schema(schema_ty.clone())),
                                    kind: ScopeObjectKind::Definition,
                                    doc: Some(schema_ty.doc),
                                },
                            )
                        }
                    }
                }
                // 2.  Build all variable types
                self.init_global_var_types(false);
            }
            None => {}
        };
    }

    /// Init global var types.
    pub(crate) fn init_global_var_types(&mut self, unique_check: bool) {
        let pkgpath = &self.ctx.pkgpath;
        match self.program.pkgs.get(pkgpath) {
            Some(modules) => {
                // 1. Scan all schema and rule type symbol
                for module in modules {
                    self.ctx.filename = module.filename.to_string();
                    for stmt in &module.body {
                        if matches!(stmt.node, ast::Stmt::TypeAlias(_)) {
                            self.stmt(stmt);
                        }
                    }
                    self.init_scope_with_stmts(&module.body, unique_check);
                }
            }
            None => {
                let pos = Position {
                    filename: self.ctx.filename.clone(),
                    line: 1,
                    column: None,
                };
                self.handler.add_error(
                    ErrorKind::CannotFindModule,
                    &[Message {
                        range: (pos.clone(), pos),
                        style: Style::Line,
                        message: format!("pkgpath {} not found in the program", self.ctx.pkgpath),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
            }
        };
    }

    fn init_scope_with_stmts(
        &mut self,
        stmts: &'ctx [ast::NodeRef<ast::Stmt>],
        unique_check: bool,
    ) {
        for stmt in stmts {
            match &stmt.node {
                ast::Stmt::Assign(assign_stmt) => {
                    self.init_scope_with_assign_stmt(assign_stmt, unique_check)
                }
                ast::Stmt::Unification(unification_stmt) => {
                    self.init_scope_with_unification_stmt(unification_stmt)
                }
                ast::Stmt::If(if_stmt) => {
                    self.init_scope_with_stmts(&if_stmt.body, unique_check);
                    self.init_scope_with_stmts(&if_stmt.orelse, unique_check);
                }
                _ => {}
            }
        }
    }

    fn init_scope_with_assign_stmt(
        &mut self,
        assign_stmt: &'ctx ast::AssignStmt,
        unique_check: bool,
    ) {
        for target in &assign_stmt.targets {
            if target.node.names.is_empty() {
                self.handler.add_compile_error(
                    "missing target in the assign statement",
                    target.get_span_pos(),
                );
                continue;
            }
            let name = &target.node.names[0].node;
            let (start, end) = target.get_span_pos();
            if self.contains_object(name) && !is_private_field(name) && unique_check {
                self.handler.add_error(
                    ErrorKind::ImmutableError,
                    &[
                        Message {
                            range: target.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                            "Can not change the value of '{}', because it was declared immutable",
                            name
                        ),
                            note: None,
                            suggested_replacement: None,
                        },
                        Message {
                            range: self
                                .scope
                                .borrow()
                                .elems
                                .get(name)
                                .unwrap()
                                .borrow()
                                .get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!("The variable '{}' is declared here", name),
                            note: Some(format!(
                                "change the variable name to '_{}' to make it mutable",
                                name
                            )),
                            suggested_replacement: None,
                        },
                    ],
                );
                continue;
            }
            let ty = if let Some(ty_annotation) = &assign_stmt.ty {
                let ty =
                    self.parse_ty_with_scope(Some(&ty_annotation), ty_annotation.get_span_pos());
                if let Some(obj) = self.scope.borrow().elems.get(name) {
                    let obj = obj.borrow();
                    if !is_upper_bound(obj.ty.clone(), ty.clone()) {
                        self.handler.add_error(
                            ErrorKind::TypeError,
                            &[
                                Message {
                                    range: target.get_span_pos(),
                                    style: Style::LineAndColumn,
                                    message: format!(
                                        "can not change the type of '{}' to {}",
                                        name,
                                        obj.ty.ty_str()
                                    ),
                                    note: None,
                                    suggested_replacement: None,
                                },
                                Message {
                                    range: obj.get_span_pos(),
                                    style: Style::LineAndColumn,
                                    message: format!("expected {}", obj.ty.ty_str()),
                                    note: None,
                                    suggested_replacement: None,
                                },
                            ],
                        );
                    }
                }
                ty
            } else if let Some(obj) = self.scope.borrow().elems.get(name) {
                obj.borrow().ty.clone()
            } else {
                self.any_ty()
            };
            self.insert_object(
                name,
                ScopeObject {
                    name: name.to_string(),
                    start,
                    end,
                    ty,
                    kind: ScopeObjectKind::Variable,
                    doc: None,
                },
            );
        }
    }

    fn init_scope_with_unification_stmt(&mut self, unification_stmt: &'ctx ast::UnificationStmt) {
        let target = &unification_stmt.target;
        if target.node.names.is_empty() {
            return;
        }
        let name = &target.node.names[0].node;
        let (start, end) = target.get_span_pos();
        let ty = self.walk_identifier_expr(&unification_stmt.value.node.name);
        self.insert_object(
            name,
            ScopeObject {
                name: name.to_string(),
                start,
                end,
                ty,
                kind: ScopeObjectKind::Variable,
                doc: None,
            },
        );
    }

    pub(crate) fn build_rule_protocol_type(
        &mut self,
        rule_stmt: &'ctx ast::RuleStmt,
    ) -> Option<Box<SchemaType>> {
        if let Some(host_name) = &rule_stmt.for_host_name {
            let ty = self.walk_identifier_expr(&host_name);
            match &ty.kind {
                TypeKind::Schema(schema_ty) if schema_ty.is_protocol && !schema_ty.is_instance => {
                    Some(Box::new(schema_ty.clone()))
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            range: host_name.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                                "invalid schema inherit object type, expect protocol, got '{}'",
                                ty.ty_str()
                            ),
                            note: None,
                            suggested_replacement: None,
                        }],
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn build_schema_protocol_type(
        &mut self,
        schema_stmt: &'ctx ast::SchemaStmt,
    ) -> Option<Box<SchemaType>> {
        if let Some(host_name) = &schema_stmt.for_host_name {
            if !schema_stmt.is_mixin {
                self.handler.add_error(
                    ErrorKind::IllegalInheritError,
                    &[Message {
                        range: host_name.get_span_pos(),
                        style: Style::LineAndColumn,
                        message: "only schema mixin can inherit from protocol".to_string(),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
                return None;
            }
            // Mixin type check with protocol
            let ty = self.walk_identifier_expr(&host_name);
            match &ty.kind {
                TypeKind::Schema(schema_ty) if schema_ty.is_protocol && !schema_ty.is_instance => {
                    Some(Box::new(schema_ty.clone()))
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            range: host_name.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                                "invalid schema inherit object type, expect protocol, got '{}'",
                                ty.ty_str()
                            ),
                            note: None,
                            suggested_replacement: None,
                        }],
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn build_schema_parent_type(
        &mut self,
        schema_stmt: &'ctx ast::SchemaStmt,
    ) -> Option<Box<SchemaType>> {
        if let Some(parent_name) = &schema_stmt.parent_name {
            let ty = self.walk_identifier_expr(&parent_name);
            match &ty.kind {
                TypeKind::Schema(schema_ty)
                    if !schema_ty.is_protocol && !schema_ty.is_mixin && !schema_ty.is_instance =>
                {
                    Some(Box::new(schema_ty.clone()))
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            range: parent_name.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                                "invalid schema inherit object type, expect schema, got '{}'",
                                ty.ty_str()
                            ),
                            note: None,
                            suggested_replacement: None,
                        }],
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn build_schema_type(
        &mut self,
        schema_stmt: &'ctx ast::SchemaStmt,
        parent_ty: Option<Box<SchemaType>>,
        protocol_ty: Option<Box<SchemaType>>,
        should_add_schema_ref: bool,
    ) -> SchemaType {
        let name = &schema_stmt.name.node;
        if RESERVED_TYPE_IDENTIFIERS.contains(&name.as_str()) {
            self.handler.add_compile_error(
                &format!(
                    "schema name '{}' cannot be the same as the built-in types ({:?})",
                    name, RESERVED_TYPE_IDENTIFIERS
                ),
                schema_stmt.name.get_span_pos(),
            );
        }
        if schema_stmt.is_protocol && !name.ends_with(PROTOCOL_SUFFIX) {
            self.handler.add_error(
                ErrorKind::CompileError,
                &[Message {
                    range: schema_stmt.name.get_span_pos(),
                    style: Style::LineAndColumn,
                    message: format!("schema protocol name must end with '{}'", PROTOCOL_SUFFIX),
                    note: None,
                    suggested_replacement: None,
                }],
            );
        }
        if schema_stmt.is_protocol && !schema_stmt.has_only_attribute_definitions() {
            self.handler.add_compile_error(
                "a protocol is only allowed to define attributes in it",
                schema_stmt.name.get_span_pos(),
            );
        }
        let parent_name = parent_ty
            .as_ref()
            .map_or("".to_string(), |ty| ty.name.clone());
        if parent_name.ends_with(MIXIN_SUFFIX) {
            self.handler.add_error(
                ErrorKind::IllegalInheritError,
                &[Message {
                    range: schema_stmt.name.get_span_pos(),
                    style: Style::LineAndColumn,
                    message: format!("mixin inheritance {} is prohibited", parent_name),
                    note: None,
                    suggested_replacement: None,
                }],
            );
        }
        let schema_attr_names = schema_stmt.get_left_identifier_list();
        let schema_attr_names: Vec<String> = schema_attr_names
            .iter()
            .map(|attr| attr.2.clone())
            .collect();
        let index_signature = if let Some(index_signature) = &schema_stmt.index_signature {
            if let Some(index_sign_name) = &index_signature.node.key_name {
                if schema_attr_names.contains(index_sign_name) {
                    self.handler.add_error(
                        ErrorKind::IndexSignatureError,
                        &[Message {
                            range: index_signature.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!("index signature attribute name '{}' cannot have the same name as schema attributes", index_sign_name),
                            note: None,
                            suggested_replacement: None,
                        }],
                    );
                }
            }
            let key_ty = self.parse_ty_str_with_scope(
                &index_signature.node.key_ty.node.to_string(),
                index_signature.node.key_ty.get_span_pos(),
            );
            let val_ty = self.parse_ty_with_scope(
                Some(&index_signature.node.value_ty),
                index_signature.node.value_ty.get_span_pos(),
            );
            if !self
                .ctx
                .ty_ctx
                .is_str_type_or_str_union_type(key_ty.clone())
            {
                self.handler.add_error(
                    ErrorKind::IndexSignatureError,
                    &[Message {
                        range: schema_stmt.name.get_span_pos(),
                        style: Style::LineAndColumn,
                        message: format!("invalid index signature key type: '{}'", key_ty.ty_str()),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
            }
            Some(Box::new(SchemaIndexSignature {
                key_name: index_signature.node.key_name.clone(),
                key_ty,
                val_ty,
                any_other: index_signature.node.any_other,
            }))
        } else {
            None
        };
        // Schema attributes
        let mut attr_obj_map: IndexMap<String, SchemaAttr> = IndexMap::default();
        let parsed_doc = parse_doc_string(
            &schema_stmt
                .doc
                .as_ref()
                .map(|doc| doc.node.clone())
                .unwrap_or_default(),
        );
        for stmt in &schema_stmt.body {
            let (name, ty, is_optional, default, decorators, range) = match &stmt.node {
                ast::Stmt::Unification(unification_stmt) => {
                    let name = unification_stmt.value.node.name.node.get_name();
                    let ty = self.parse_ty_str_with_scope(&name, stmt.get_span_pos());
                    let is_optional = false;
                    let default = if self.options.resolve_val {
                        print_schema_expr(&unification_stmt.value.node)
                    } else {
                        "".to_string()
                    };
                    (
                        unification_stmt.target.node.get_name(),
                        ty,
                        is_optional,
                        Some(default),
                        vec![],
                        stmt.get_span_pos(),
                    )
                }
                ast::Stmt::SchemaAttr(schema_attr) => {
                    let name = schema_attr.name.node.clone();
                    let ty = self
                        .parse_ty_with_scope(Some(&schema_attr.ty), schema_attr.ty.get_span_pos());
                    let is_optional = schema_attr.is_optional;
                    let default = schema_attr.value.as_ref().map(|v| {
                        if self.options.resolve_val {
                            print_ast_node(ASTNode::Expr(v))
                        } else {
                            "".to_string()
                        }
                    });
                    // Schema attribute decorators
                    let decorators = self.resolve_decorators(
                        &schema_attr.decorators,
                        DecoratorTarget::Attribute,
                        &name,
                    );
                    (
                        name,
                        ty,
                        is_optional,
                        default,
                        decorators,
                        stmt.get_span_pos(),
                    )
                }
                _ => continue,
            };
            let base_attr_ty = match parent_ty {
                Some(ref ty) => ty.get_type_of_attr(&name).map_or(self.any_ty(), |ty| ty),
                None => self.any_ty(),
            };
            if !attr_obj_map.contains_key(&name) {
                let existed_attr = parent_ty.as_ref().and_then(|ty| ty.get_obj_of_attr(&name));
                let doc_str = parsed_doc.attrs.iter().find_map(|attr| {
                    if attr.name == name {
                        Some(attr.desc.join("\n"))
                    } else {
                        None
                    }
                });
                attr_obj_map.insert(
                    name.clone(),
                    SchemaAttr {
                        is_optional: existed_attr.map_or(is_optional, |attr| attr.is_optional),
                        has_default: default.is_some(),
                        default,
                        ty: ty.clone(),
                        range: range.clone(),
                        doc: doc_str,
                        decorators,
                    },
                );
            }
            if !is_upper_bound(attr_obj_map.get(&name).unwrap().ty.clone(), ty.clone())
                || !is_upper_bound(base_attr_ty.clone(), ty.clone())
            {
                self.handler.add_type_error(
                    &format!(
                        "can't change schema field type of '{}' from {} to {}",
                        name,
                        attr_obj_map.get(&name).unwrap().ty.clone().ty_str(),
                        ty.ty_str()
                    ),
                    stmt.get_span_pos(),
                );
            }
            if is_optional && !attr_obj_map.get(&name).unwrap().is_optional {
                self.handler.add_type_error(
                    &format!(
                        "can't change the required schema attribute of '{}' to optional",
                        name
                    ),
                    stmt.get_span_pos(),
                );
            }
            if let Some(ref index_signature_obj) = index_signature {
                if !index_signature_obj.any_other
                    && !is_upper_bound(index_signature_obj.val_ty.clone(), ty.clone())
                {
                    self.handler.add_error(
                        ErrorKind::IndexSignatureError,
                        &[Message {
                            range: stmt.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!("the type '{}' of schema attribute '{}' does not meet the index signature definition {}", ty.ty_str(), name, index_signature_obj.ty_str()),
                            note: None,
                            suggested_replacement: None,
                        }],
                    );
                }
            }
        }
        // Mixin types
        let mut mixin_types: Vec<SchemaType> = vec![];
        for mixin in &schema_stmt.mixins {
            let mixin_names = &mixin.node.get_names();
            if !mixin_names[mixin_names.len() - 1].ends_with(MIXIN_SUFFIX) {
                self.handler.add_error(
                    ErrorKind::NameError,
                    &[Message {
                        range: mixin.get_span_pos(),
                        style: Style::LineAndColumn,
                        message: format!(
                            "a valid mixin name should end with 'Mixin', got '{}'",
                            mixin_names[mixin_names.len() - 1]
                        ),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
            }
            let ty = self.walk_identifier_expr(&mixin);
            let mixin_ty = match &ty.kind {
                TypeKind::Schema(schema_ty)
                    if !schema_ty.is_protocol && schema_ty.is_mixin && !schema_ty.is_instance =>
                {
                    Some(schema_ty.clone())
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            range: mixin.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                                "illegal schema mixin object type, expected mixin, got '{}'",
                                ty.ty_str()
                            ),
                            note: None,
                            suggested_replacement: None,
                        }],
                    );
                    None
                }
            };

            if let Some(mixin_ty) = mixin_ty {
                for (name, attr) in &mixin_ty.attrs {
                    if !attr_obj_map.contains_key(name) {
                        attr_obj_map.insert(name.to_string(), attr.clone());
                    }
                }
                mixin_types.push(mixin_ty);
            }
        }
        // Params
        let mut params: Vec<Parameter> = vec![];
        if let Some(args) = &schema_stmt.args {
            for (i, para) in args.node.args.iter().enumerate() {
                let name = para.node.get_name();
                if schema_attr_names.contains(&name) {
                    self.handler.add_compile_error(
                        &format!(
                            "Unexpected parameter name '{}' with the same name as the schema attribute",
                            name
                        ),
                        para.get_span_pos(),
                    );
                }
                let ty = args.node.get_arg_type_node(i);
                let ty = self.parse_ty_with_scope(ty, para.get_span_pos());
                params.push(Parameter {
                    name,
                    ty: ty.clone(),
                    has_default: args.node.defaults.get(i).map_or(false, |arg| arg.is_some()),
                });
            }
        }
        let schema_runtime_ty = kclvm_runtime::schema_runtime_type(name, &self.ctx.pkgpath);
        if should_add_schema_ref {
            if let Some(ref parent_ty) = parent_ty {
                let parent_schema_runtime_ty =
                    kclvm_runtime::schema_runtime_type(&parent_ty.name, &parent_ty.pkgpath);
                self.ctx
                    .ty_ctx
                    .add_dependencies(&schema_runtime_ty, &parent_schema_runtime_ty);
                if self.ctx.ty_ctx.is_cyclic() {
                    self.handler.add_compile_error(
                        &format!(
                            "There is a circular reference between schema {} and {}",
                            name, parent_ty.name,
                        ),
                        schema_stmt.get_span_pos(),
                    );
                }
            }
        }
        let decorators = self.resolve_decorators(
            &schema_stmt.decorators,
            DecoratorTarget::Schema,
            &schema_stmt.name.node,
        );
        let schema_ty = SchemaType {
            name: schema_stmt.name.node.clone(),
            pkgpath: self.ctx.pkgpath.clone(),
            filename: self.ctx.filename.clone(),
            doc: parsed_doc.summary.clone(),
            examples: parsed_doc.examples,
            is_instance: false,
            is_mixin: schema_stmt.is_mixin,
            is_protocol: schema_stmt.is_protocol,
            is_rule: false,
            base: parent_ty,
            protocol: protocol_ty,
            mixins: mixin_types,
            attrs: attr_obj_map,
            func: Box::new(FunctionType {
                doc: parsed_doc.summary.clone(),
                params,
                self_ty: None,
                return_ty: Arc::new(Type::ANY),
                is_variadic: false,
                kw_only_index: None,
            }),
            index_signature,
            decorators,
        };
        self.ctx
            .schema_mapping
            .insert(schema_runtime_ty, Arc::new(RefCell::new(schema_ty.clone())));
        schema_ty
    }

    pub(crate) fn build_rule_type(
        &mut self,
        rule_stmt: &'ctx ast::RuleStmt,
        protocol_ty: Option<Box<SchemaType>>,
        should_add_schema_ref: bool,
    ) -> SchemaType {
        let name = &rule_stmt.name.node;
        if RESERVED_TYPE_IDENTIFIERS.contains(&name.as_str()) {
            self.handler.add_compile_error(
                &format!(
                    "rule name '{}' cannot be the same as the built-in types ({:?})",
                    name, RESERVED_TYPE_IDENTIFIERS
                ),
                rule_stmt.name.get_span_pos(),
            );
        }
        // Parent types
        let mut parent_types: Vec<SchemaType> = vec![];
        for rule in &rule_stmt.parent_rules {
            let ty = self.walk_identifier_expr(&rule);
            let parent_ty = match &ty.kind {
                TypeKind::Schema(schema_ty) if schema_ty.is_rule && !schema_ty.is_instance => {
                    Some(schema_ty.clone())
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            range: rule.get_span_pos(),
                            style: Style::LineAndColumn,
                            message: format!("illegal rule type '{}'", ty.ty_str()),
                            note: None,
                            suggested_replacement: None,
                        }],
                    );
                    None
                }
            };
            if let Some(parent_ty) = parent_ty {
                parent_types.push(parent_ty);
            }
        }
        // Params
        let mut params: Vec<Parameter> = vec![];
        if let Some(args) = &rule_stmt.args {
            for (i, para) in args.node.args.iter().enumerate() {
                let name = para.node.get_name();
                let ty = args.node.get_arg_type_node(i);
                let ty = self.parse_ty_with_scope(ty, para.get_span_pos());
                params.push(Parameter {
                    name,
                    ty: ty.clone(),
                    has_default: args.node.defaults.get(i).is_some(),
                });
            }
        }
        if should_add_schema_ref {
            let schema_runtime_ty = kclvm_runtime::schema_runtime_type(name, &self.ctx.pkgpath);
            for parent_ty in &parent_types {
                let parent_schema_runtime_ty =
                    kclvm_runtime::schema_runtime_type(&parent_ty.name, &parent_ty.pkgpath);
                self.ctx
                    .ty_ctx
                    .add_dependencies(&schema_runtime_ty, &parent_schema_runtime_ty);
                if self.ctx.ty_ctx.is_cyclic() {
                    self.handler.add_compile_error(
                        &format!(
                            "There is a circular reference between rule {} and {}",
                            name, parent_ty.name,
                        ),
                        rule_stmt.get_span_pos(),
                    );
                }
            }
        }
        let decorators = self.resolve_decorators(
            &rule_stmt.decorators,
            DecoratorTarget::Schema,
            &rule_stmt.name.node,
        );

        let parsed_doc = parse_doc_string(
            &rule_stmt
                .doc
                .as_ref()
                .map(|doc| doc.node.clone())
                .unwrap_or_default(),
        );
        let index_signature = match &protocol_ty {
            Some(ty) => ty.index_signature.clone(),
            None => None,
        };
        SchemaType {
            name: rule_stmt.name.node.clone(),
            pkgpath: self.ctx.pkgpath.clone(),
            filename: self.ctx.filename.clone(),
            doc: parsed_doc.summary.clone(),
            examples: parsed_doc.examples,
            is_instance: false,
            is_mixin: false,
            is_protocol: false,
            is_rule: true,
            base: None,
            protocol: protocol_ty,
            mixins: parent_types,
            attrs: IndexMap::default(),
            func: Box::new(FunctionType {
                doc: rule_stmt
                    .doc
                    .as_ref()
                    .map(|doc| doc.node.clone())
                    .unwrap_or_default(),
                params,
                self_ty: None,
                return_ty: Arc::new(Type::ANY),
                is_variadic: false,
                kw_only_index: None,
            }),
            index_signature,
            decorators,
        }
    }
}
