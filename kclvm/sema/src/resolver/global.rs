use std::cell::RefCell;
use std::rc::Rc;

use crate::info::is_private_field;
use crate::resolver::Resolver;
use crate::ty::{
    is_upper_bound, DecoratorTarget, FunctionType, Parameter, SchemaAttr, SchemaIndexSignature,
    SchemaType, Type, TypeKind, RESERVED_TYPE_IDENTIFIERS,
};
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::*;

use super::scope::{ScopeObject, ScopeObjectKind};
use crate::resolver::pos::GetPos;

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
                                &schema_stmt.doc,
                                schema_stmt.is_mixin,
                                schema_stmt.is_protocol,
                                false,
                            ),
                            ast::Stmt::Rule(rule_stmt) => {
                                (&rule_stmt.name.node, &rule_stmt.doc, false, false, true)
                            }
                            _ => continue,
                        };
                        if self.contains_object(name) {
                            self.handler.add_error(
                                ErrorKind::UniqueKeyError,
                                &[Message {
                                    pos: start.clone(),
                                    style: Style::LineAndColumn,
                                    message: format!("unique key error name '{}'", name),
                                    note: None,
                                }],
                            );
                            continue;
                        }
                        let schema_ty = SchemaType {
                            name: name.to_string(),
                            pkgpath: self.ctx.pkgpath.clone(),
                            filename: self.ctx.filename.clone(),
                            doc: doc.to_string(),
                            is_instance: false,
                            is_mixin,
                            is_protocol,
                            is_rule,
                            base: None,
                            protocol: None,
                            mixins: vec![],
                            attrs: IndexMap::default(),
                            func: Box::new(FunctionType {
                                doc: doc.to_string(),
                                params: vec![],
                                self_ty: None,
                                return_ty: Rc::new(Type::VOID),
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
                                ty: Rc::new(Type::schema(schema_ty)),
                                kind: ScopeObjectKind::Definition,
                                used: false,
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
                                    ty: Rc::new(Type::schema(schema_ty)),
                                    kind: ScopeObjectKind::Definition,
                                    used: false,
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
                self.handler.add_error(
                    ErrorKind::CannotFindModule,
                    &[Message {
                        pos: Position {
                            filename: self.ctx.filename.clone(),
                            line: 1,
                            column: None,
                        },
                        style: Style::Line,
                        message: format!("pkgpath {} not found in the program", self.ctx.pkgpath),
                        note: None,
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
                    self.init_scope_with_unification_stmt(unification_stmt, unique_check)
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
            let name = &target.node.names[0];
            let (start, end) = target.get_span_pos();
            if self.contains_object(name) && !is_private_field(name) && unique_check {
                self.handler.add_error(
                    ErrorKind::ImmutableError,
                    &[
                        Message {
                            pos: start.clone(),
                            style: Style::LineAndColumn,
                            message: format!(
                            "Can not change the value of '{}', because it was declared immutable",
                            name
                        ),
                            note: None,
                        },
                        Message {
                            pos: self
                                .scope
                                .borrow()
                                .elems
                                .get(name)
                                .unwrap()
                                .borrow()
                                .start
                                .clone(),
                            style: Style::LineAndColumn,
                            message: format!("The variable '{}' is declared here firstly", name),
                            note: Some(format!(
                                "change the variable name to '_{}' to make it mutable",
                                name
                            )),
                        },
                    ],
                );
                continue;
            }
            let ty = if let Some(ty_annotation) = &assign_stmt.ty {
                let ty = &ty_annotation.node;
                let ty = self.parse_ty_with_scope(ty, ty_annotation.get_pos());
                if let Some(obj) = self.scope.borrow().elems.get(name) {
                    let obj = obj.borrow();
                    if !is_upper_bound(obj.ty.clone(), ty.clone()) {
                        self.handler.add_error(
                            ErrorKind::TypeError,
                            &[
                                Message {
                                    pos: start.clone(),
                                    style: Style::LineAndColumn,
                                    message: format!(
                                        "can not change the type of '{}' to {}",
                                        name,
                                        obj.ty.ty_str()
                                    ),
                                    note: None,
                                },
                                Message {
                                    pos: obj.start.clone(),
                                    style: Style::LineAndColumn,
                                    message: format!("expect {}", obj.ty.ty_str()),
                                    note: None,
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
                    used: false,
                },
            );
        }
    }

    fn init_scope_with_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
        unique_check: bool,
    ) {
        let target = &unification_stmt.target;
        let name = &target.node.names[0];
        let (start, end) = target.get_span_pos();
        if self.contains_object(name) && !is_private_field(name) && unique_check {
            self.handler.add_error(
                ErrorKind::ImmutableError,
                &[
                    Message {
                        pos: start,
                        style: Style::LineAndColumn,
                        message: format!(
                            "Can not change the value of '{}', because it was declared immutable",
                            name
                        ),
                        note: None,
                    },
                    Message {
                        pos: self
                            .scope
                            .borrow()
                            .elems
                            .get(name)
                            .unwrap()
                            .borrow()
                            .start
                            .clone(),
                        style: Style::LineAndColumn,
                        message: format!("The variable '{}' is declared here firstly", name),
                        note: Some(format!(
                            "change the variable name to '_{}' to make it mutable",
                            name
                        )),
                    },
                ],
            );
            return;
        }
        let ty = self.walk_identifier(&unification_stmt.value.node.name.node);
        self.insert_object(
            name,
            ScopeObject {
                name: name.to_string(),
                start,
                end,
                ty,
                kind: ScopeObjectKind::Variable,
                used: false,
            },
        );
    }

    pub(crate) fn build_rule_protocol_type(
        &mut self,
        rule_stmt: &'ctx ast::RuleStmt,
    ) -> Option<Box<SchemaType>> {
        if let Some(host_name) = &rule_stmt.for_host_name {
            let ty = self.walk_identifier(&host_name.node);
            match &ty.kind {
                TypeKind::Schema(schema_ty) if schema_ty.is_protocol && !schema_ty.is_instance => {
                    Some(Box::new(schema_ty.clone()))
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            pos: host_name.get_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                                "invalid schema inherit object type, expect protocol, got '{}'",
                                ty.ty_str()
                            ),
                            note: None,
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
                        pos: host_name.get_pos(),
                        style: Style::LineAndColumn,
                        message: "only schema mixin can inherit from protocol".to_string(),
                        note: None,
                    }],
                );
                return None;
            }
            // Mixin type check with protocol
            let ty = self.walk_identifier(&host_name.node);
            match &ty.kind {
                TypeKind::Schema(schema_ty) if schema_ty.is_protocol && !schema_ty.is_instance => {
                    Some(Box::new(schema_ty.clone()))
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            pos: host_name.get_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                                "invalid schema inherit object type, expect protocol, got '{}'",
                                ty.ty_str()
                            ),
                            note: None,
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
            let ty = self.walk_identifier(&parent_name.node);
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
                            pos: parent_name.get_pos(),
                            style: Style::LineAndColumn,
                            message: format!(
                                "invalid schema inherit object type, expect schema, got '{}'",
                                ty.ty_str()
                            ),
                            note: None,
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
        let pos = schema_stmt.name.get_end_pos();
        if RESERVED_TYPE_IDENTIFIERS.contains(&name.as_str()) {
            self.handler.add_compile_error(
                &format!(
                    "schema name '{}' cannot be the same as the built-in types ({:?})",
                    name, RESERVED_TYPE_IDENTIFIERS
                ),
                pos.clone(),
            );
        }
        if schema_stmt.is_protocol && !name.ends_with(PROTOCOL_SUFFIX) {
            self.handler.add_error(
                ErrorKind::CompileError,
                &[Message {
                    pos: pos.clone(),
                    style: Style::LineAndColumn,
                    message: format!("schema protocol name must end with '{}'", PROTOCOL_SUFFIX),
                    note: None,
                }],
            );
        }
        if schema_stmt.is_protocol && !schema_stmt.has_only_attribute_definitions() {
            self.handler.add_compile_error(
                "a protocol is only allowed to define attributes in it",
                pos.clone(),
            );
        }
        let parent_name = parent_ty
            .as_ref()
            .map_or("".to_string(), |ty| ty.name.clone());
        if parent_name.ends_with(MIXIN_SUFFIX) {
            self.handler.add_error(
                ErrorKind::IllegalInheritError,
                &[Message {
                    pos: pos.clone(),
                    style: Style::LineAndColumn,
                    message: format!("mixin inheritance {} is prohibited", parent_name),
                    note: None,
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
                            pos: index_signature.get_pos(),
                            style: Style::LineAndColumn,
                            message: format!("index signature attribute name '{}' cannot have the same name as schema attributes", index_sign_name),
                            note: None,
                        }],
                    );
                }
            }
            let key_ty = self.parse_ty_str_with_scope(
                &index_signature.node.key_type.node,
                index_signature.node.key_type.get_pos(),
            );
            let val_ty = self.parse_ty_with_scope(
                &index_signature.node.value_ty.node,
                index_signature.node.value_type.get_pos(),
            );
            if !self
                .ctx
                .ty_ctx
                .is_str_type_or_str_union_type(key_ty.clone())
            {
                self.handler.add_error(
                    ErrorKind::IndexSignatureError,
                    &[Message {
                        pos: pos.clone(),
                        style: Style::LineAndColumn,
                        message: format!("invalid index signature key type: '{}'", key_ty.ty_str()),
                        note: None,
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
        attr_obj_map.insert(
            kclvm_runtime::SCHEMA_SETTINGS_ATTR_NAME.to_string(),
            SchemaAttr {
                is_optional: true,
                has_default: false,
                ty: Type::dict_ref(self.str_ty(), self.any_ty()),
                pos: Position {
                    filename: self.ctx.filename.clone(),
                    line: pos.line,
                    column: pos.column,
                },
            },
        );
        for stmt in &schema_stmt.body {
            let pos = stmt.get_pos();
            let (name, ty, is_optional, has_default) = match &stmt.node {
                ast::Stmt::Unification(unification_stmt) => {
                    let name = unification_stmt.value.node.name.node.get_name();
                    let ty = self.parse_ty_str_with_scope(&name, pos.clone());
                    let is_optional = true;
                    let has_default = true;
                    (
                        unification_stmt.target.node.get_name(),
                        ty,
                        is_optional,
                        has_default,
                    )
                }
                ast::Stmt::SchemaAttr(schema_attr) => {
                    let name = schema_attr.name.node.clone();
                    let ty = self.parse_ty_with_scope(
                        &schema_attr.ty.node.clone(),
                        schema_attr.ty.get_pos(),
                    );
                    let is_optional = schema_attr.is_optional;
                    let has_default = schema_attr.value.is_some();
                    (name, ty, is_optional, has_default)
                }
                _ => continue,
            };
            let base_attr_ty = match parent_ty {
                Some(ref ty) => ty.get_type_of_attr(&name).map_or(self.any_ty(), |ty| ty),
                None => self.any_ty(),
            };
            if !attr_obj_map.contains_key(&name) {
                let existed_attr = parent_ty.as_ref().and_then(|ty| ty.get_obj_of_attr(&name));
                attr_obj_map.insert(
                    name.clone(),
                    SchemaAttr {
                        is_optional: existed_attr.map_or(is_optional, |attr| attr.is_optional),
                        has_default,
                        ty: ty.clone(),
                        pos: pos.clone(),
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
                    pos.clone(),
                );
            }
            if is_optional && !attr_obj_map.get(&name).unwrap().is_optional {
                self.handler.add_type_error(
                    &format!(
                        "can't change the required schema attribute of '{}' to optional",
                        name
                    ),
                    pos.clone(),
                );
            }
            if let Some(ref index_signature_obj) = index_signature {
                if !index_signature_obj.any_other
                    && !is_upper_bound(index_signature_obj.val_ty.clone(), ty.clone())
                {
                    self.handler.add_error(
                        ErrorKind::IndexSignatureError,
                        &[Message {
                            pos: pos.clone(),
                            style: Style::LineAndColumn,
                            message: format!("the type '{}' of schema attribute '{}' does not meet the index signature definition {}", ty.ty_str(), name, index_signature_obj.ty_str()),
                            note: None,
                        }],
                    );
                }
            }
        }
        // Mixin types
        let mut mixin_types: Vec<SchemaType> = vec![];
        for mixin in &schema_stmt.mixins {
            let mixin_names = &mixin.node.names;
            if !mixin_names[mixin_names.len() - 1].ends_with(MIXIN_SUFFIX) {
                self.handler.add_error(
                    ErrorKind::NameError,
                    &[Message {
                        pos: pos.clone(),
                        style: Style::LineAndColumn,
                        message: format!(
                            "a valid mixin name should end with 'Mixin', got '{}'",
                            mixin_names[mixin_names.len() - 1]
                        ),
                        note: None,
                    }],
                );
            }
            let ty = self.walk_identifier(&mixin.node);
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
                            pos: mixin.get_pos(),
                            style: Style::LineAndColumn,
                            message: format!("illegal schema mixin object type '{}'", ty.ty_str()),
                            note: None,
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
                let pos = para.get_pos();
                if schema_attr_names.contains(&name) {
                    self.handler.add_compile_error(
                        &format!(
                            "Unexpected parameter name '{}' with the same name as the schema attribute",
                            name
                        ),
                        pos.clone(),
                    );
                }
                let ty = args.node.get_arg_type(i);
                let ty = self.parse_ty_with_scope(&ty, pos);
                params.push(Parameter {
                    name,
                    ty: ty.clone(),
                    has_default: args.node.defaults.get(i).is_some(),
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
                        schema_stmt.get_pos(),
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
            doc: schema_stmt.doc.clone(),
            is_instance: false,
            is_mixin: schema_stmt.is_mixin,
            is_protocol: schema_stmt.is_protocol,
            is_rule: false,
            base: parent_ty,
            protocol: protocol_ty,
            mixins: mixin_types,
            attrs: attr_obj_map,
            func: Box::new(FunctionType {
                doc: schema_stmt.doc.clone(),
                params,
                self_ty: None,
                return_ty: Rc::new(Type::ANY),
                is_variadic: false,
                kw_only_index: None,
            }),
            index_signature,
            decorators,
        };
        self.ctx
            .schema_mapping
            .insert(schema_runtime_ty, Rc::new(RefCell::new(schema_ty.clone())));
        schema_ty
    }

    pub(crate) fn build_rule_type(
        &mut self,
        rule_stmt: &'ctx ast::RuleStmt,
        protocol_ty: Option<Box<SchemaType>>,
        should_add_schema_ref: bool,
    ) -> SchemaType {
        let name = &rule_stmt.name.node;
        let pos = rule_stmt.name.get_end_pos();
        if RESERVED_TYPE_IDENTIFIERS.contains(&name.as_str()) {
            self.handler.add_compile_error(
                &format!(
                    "rule name '{}' cannot be the same as the built-in types ({:?})",
                    name, RESERVED_TYPE_IDENTIFIERS
                ),
                pos,
            );
        }
        // Parent types
        let mut parent_types: Vec<SchemaType> = vec![];
        for rule in &rule_stmt.parent_rules {
            let ty = self.walk_identifier(&rule.node);
            let parent_ty = match &ty.kind {
                TypeKind::Schema(schema_ty) if schema_ty.is_rule && !schema_ty.is_instance => {
                    Some(schema_ty.clone())
                }
                _ => {
                    self.handler.add_error(
                        ErrorKind::IllegalInheritError,
                        &[Message {
                            pos: rule.get_pos(),
                            style: Style::LineAndColumn,
                            message: format!("illegal rule type '{}'", ty.ty_str()),
                            note: None,
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
                let pos = para.get_pos();
                let ty = args.node.get_arg_type(i);
                let ty = self.parse_ty_with_scope(&ty, pos);
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
                        rule_stmt.get_pos(),
                    );
                }
            }
        }
        let decorators = self.resolve_decorators(
            &rule_stmt.decorators,
            DecoratorTarget::Schema,
            &rule_stmt.name.node,
        );
        SchemaType {
            name: rule_stmt.name.node.clone(),
            pkgpath: self.ctx.pkgpath.clone(),
            filename: self.ctx.filename.clone(),
            doc: rule_stmt.doc.clone(),
            is_instance: false,
            is_mixin: false,
            is_protocol: false,
            is_rule: true,
            base: None,
            protocol: protocol_ty,
            mixins: parent_types,
            attrs: IndexMap::default(),
            func: Box::new(FunctionType {
                doc: rule_stmt.doc.clone(),
                params,
                self_ty: None,
                return_ty: Rc::new(Type::ANY),
                is_variadic: false,
                kw_only_index: None,
            }),
            index_signature: None,
            decorators,
        }
    }
}
