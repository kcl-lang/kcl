use indexmap::{IndexMap, IndexSet};
use kclvm_ast::walker::MutSelfMutWalker;
use kclvm_ast::{ast, walk_if_mut};

const NAME_NONE_BUCKET_KEY: &str = "$name_none";

#[derive(Debug, Default)]
struct ConfigNestAttrTransformer;

impl ConfigNestAttrTransformer {
    pub fn walk_config_entry(&mut self, config_entry: &mut Box<ast::Node<ast::ConfigEntry>>) {
        if let Some(key) = config_entry.node.key.as_mut() {
            if let ast::Expr::Identifier(identifier) = &mut key.node {
                if identifier.names.len() > 1 {
                    let mut names = identifier.names.clone();
                    let names = &mut names[1..];
                    names.reverse();
                    identifier.names = vec![identifier.names[0].clone()];

                    let mut value = config_entry.node.value.clone();
                    for (i, name) in names.iter().enumerate() {
                        let is_last_item = i == 0;
                        let name_node = ast::Identifier {
                            names: vec![name.clone()],
                            pkgpath: "".to_string(),
                            ctx: ast::ExprContext::Load,
                        };
                        let entry_value = ast::ConfigEntry {
                            key: Some(Box::new(ast::Node::new(
                                ast::Expr::Identifier(name_node),
                                key.filename.clone(),
                                key.line,
                                key.column,
                                key.end_line,
                                key.end_column,
                            ))),
                            value: value.clone(),
                            operation: if is_last_item {
                                config_entry.node.operation.clone()
                            } else {
                                ast::ConfigEntryOperation::Union
                            },
                            insert_index: -1,
                        };
                        let config_expr = ast::ConfigExpr {
                            items: vec![Box::new(ast::Node::new(
                                entry_value,
                                key.filename.clone(),
                                key.line,
                                key.column,
                                key.end_line,
                                key.end_column,
                            ))],
                        };
                        value = Box::new(ast::Node::new(
                            ast::Expr::Config(config_expr),
                            value.filename.clone(),
                            value.line,
                            value.column,
                            value.end_line,
                            value.end_column,
                        ))
                    }
                    config_entry.node.value = value;
                    config_entry.node.operation = ast::ConfigEntryOperation::Union;
                }
            }
        }
    }
}

impl<'ctx> MutSelfMutWalker<'ctx> for ConfigNestAttrTransformer {
    fn walk_config_expr(&mut self, config_expr: &'ctx mut ast::ConfigExpr) {
        for config_entry in config_expr.items.iter_mut() {
            self.walk_config_entry(config_entry);
            self.walk_expr(&mut config_entry.node.value.node);
        }
    }
    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx mut ast::ConfigIfEntryExpr,
    ) {
        for config_entry in config_if_entry_expr.items.iter_mut() {
            self.walk_config_entry(config_entry);
            self.walk_expr(&mut config_entry.node.value.node);
        }
        walk_if_mut!(self, walk_expr, config_if_entry_expr.orelse);
    }
}

#[derive(Debug)]
struct ConfigMergeTransformer {}

#[derive(Debug)]
enum ConfigMergeKind {
    Override,
    Union,
}

impl ConfigMergeTransformer {
    pub fn merge(&mut self, program: &mut ast::Program) {
        // {name: (filename, module index in main package, statement index in the module body, kind)}
        // module index is to prevent same filename in main package
        let mut name_declaration_mapping: IndexMap<
            String,
            Vec<(String, usize, usize, ConfigMergeKind)>,
        > = IndexMap::default();
        // 1. Collect merged config
        if let Some(modules) = program.pkgs.get_mut(kclvm_ast::MAIN_PKG) {
            for (module_id, module) in modules.iter_mut().enumerate() {
                for (i, stmt) in module.body.iter_mut().enumerate() {
                    match &mut stmt.node {
                        ast::Stmt::Unification(unification_stmt) => {
                            let name = &unification_stmt.target.node.names[0];
                            match name_declaration_mapping.get_mut(name) {
                                Some(declarations) => declarations.push((
                                    module.filename.to_string(),
                                    module_id,
                                    i,
                                    ConfigMergeKind::Union,
                                )),
                                None => {
                                    name_declaration_mapping.insert(
                                        name.to_string(),
                                        vec![(
                                            module.filename.to_string(),
                                            module_id,
                                            i,
                                            ConfigMergeKind::Union,
                                        )],
                                    );
                                }
                            }
                        }
                        ast::Stmt::Assign(assign_stmt) => {
                            if let ast::Expr::Schema(_) = assign_stmt.value.node {
                                for target in &assign_stmt.targets {
                                    if target.node.names.len() == 1 {
                                        let name = &target.node.names[0];
                                        match name_declaration_mapping.get_mut(name) {
                                            Some(declarations) => declarations.push((
                                                module.filename.to_string(),
                                                module_id,
                                                i,
                                                ConfigMergeKind::Override,
                                            )),
                                            None => {
                                                name_declaration_mapping.insert(
                                                    name.to_string(),
                                                    vec![(
                                                        module.filename.to_string(),
                                                        module_id,
                                                        i,
                                                        ConfigMergeKind::Override,
                                                    )],
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // 2. Merge config
        for (_, index_list) in &name_declaration_mapping {
            let index_len = index_list.len();
            if index_len > 1 {
                let (filename, merged_id, merged_index, merged_kind) = index_list.last().unwrap();
                let mut items: Vec<ast::NodeRef<ast::ConfigEntry>> = vec![];
                for (merged_filename, merged_id, index, kind) in index_list {
                    if let Some(modules) = program.pkgs.get_mut(kclvm_ast::MAIN_PKG) {
                        for (module_id, module) in modules.iter_mut().enumerate() {
                            if &module.filename == merged_filename && module_id == *merged_id {
                                let stmt = module.body.get_mut(*index).unwrap();
                                match &mut stmt.node {
                                    ast::Stmt::Unification(unification_stmt)
                                        if matches!(kind, ConfigMergeKind::Union) =>
                                    {
                                        if let ast::Expr::Config(config_expr) =
                                            &mut unification_stmt.value.node.config.node
                                        {
                                            let mut config_items = config_expr.items.clone();
                                            items.append(&mut config_items);
                                        }
                                    }
                                    ast::Stmt::Assign(assign_stmt)
                                        if matches!(kind, ConfigMergeKind::Override) =>
                                    {
                                        if let ast::Expr::Schema(schema_expr) =
                                            &mut assign_stmt.value.node
                                        {
                                            if let ast::Expr::Config(config_expr) =
                                                &mut schema_expr.config.node
                                            {
                                                let mut config_items = config_expr.items.clone();
                                                items.append(&mut config_items);
                                            }
                                        }
                                    }
                                    _ => {
                                        bug!("mismatch ast node and config merge kind: {:?}", kind)
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(modules) = program.pkgs.get_mut(kclvm_ast::MAIN_PKG) {
                    for (module_id, module) in modules.iter_mut().enumerate() {
                        if &module.filename == filename && module_id == *merged_id {
                            if let Some(stmt) = module.body.get_mut(*merged_index) {
                                match &mut stmt.node {
                                    ast::Stmt::Unification(unification_stmt)
                                        if matches!(merged_kind, ConfigMergeKind::Union) =>
                                    {
                                        if let ast::Expr::Config(config_expr) =
                                            &mut unification_stmt.value.node.config.node
                                        {
                                            config_expr.items = unify_config_entries(&items);
                                        }
                                    }
                                    ast::Stmt::Assign(assign_stmt)
                                        if matches!(merged_kind, ConfigMergeKind::Override) =>
                                    {
                                        if let ast::Expr::Schema(schema_expr) =
                                            &mut assign_stmt.value.node
                                        {
                                            if let ast::Expr::Config(config_expr) =
                                                &mut schema_expr.config.node
                                            {
                                                config_expr.items = unify_config_entries(&items);
                                            }
                                        }
                                    }
                                    _ => bug!(
                                        "mismatch ast node and config merge kind: {:?}",
                                        merged_kind
                                    ),
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        // 3. Delete redundant config.
        if let Some(modules) = program.pkgs.get_mut(kclvm_ast::MAIN_PKG) {
            for (i, module) in modules.iter_mut().enumerate() {
                let mut delete_index_set: IndexSet<usize> = IndexSet::default();
                for (_, index_list) in &name_declaration_mapping {
                    let index_len = index_list.len();
                    if index_len > 1 {
                        for (filename, module_id, index, _) in &index_list[..index_len - 1] {
                            // Use module filename and index to prevent the same compile filenames
                            // in the main package.
                            if &module.filename == filename && i == *module_id {
                                delete_index_set.insert(*index);
                            }
                        }
                    }
                }
                let mut body: Vec<(usize, &ast::NodeRef<ast::Stmt>)> =
                    module.body.iter().enumerate().collect();
                body.retain(|(idx, _)| !delete_index_set.contains(idx));
                module.body = body
                    .iter()
                    .map(|(_, stmt)| (*stmt).clone())
                    .collect::<Vec<ast::NodeRef<ast::Stmt>>>();
            }
        }
    }
}

/// Unify config entries.
fn unify_config_entries(
    entries: &[ast::NodeRef<ast::ConfigEntry>],
) -> Vec<ast::NodeRef<ast::ConfigEntry>> {
    // Using bucket map to check unique/merge option and store values
    let mut bucket: IndexMap<String, Vec<ast::NodeRef<ast::ConfigEntry>>> = IndexMap::new();
    for entry in entries {
        let name = match &entry.node.key {
            Some(key) => match &key.node {
                ast::Expr::Identifier(identifier) => identifier.get_name(),
                ast::Expr::StringLit(string_lit) => string_lit.value.clone(),
                _ => NAME_NONE_BUCKET_KEY.to_string(),
            },
            None => NAME_NONE_BUCKET_KEY.to_string(),
        };
        let entry = entry.clone();
        match bucket.get_mut(&name) {
            Some(values) => values.push(entry),
            None => {
                let values = vec![entry];
                bucket.insert(name, values);
            }
        }
    }
    let mut entries = vec![];
    for (key, items) in bucket.iter_mut() {
        if key == NAME_NONE_BUCKET_KEY {
            entries.append(items);
        } else {
            let mut schema_index = None;
            for (i, item) in items.iter().enumerate() {
                if let ast::Expr::Schema(_) = item.node.value.node {
                    schema_index = Some(i);
                    break;
                }
            }
            match schema_index {
                Some(index) => {
                    let mut merged_schema = items[index].as_ref().clone();
                    for (i, item) in items.iter().enumerate() {
                        match &item.node.value.node {
                            ast::Expr::Schema(item_schema_expr) => {
                                if let ast::Expr::Schema(schema_expr) =
                                    &mut merged_schema.node.value.node
                                {
                                    if let ast::Expr::Config(schema_config) =
                                        &mut schema_expr.config.node
                                    {
                                        if let ast::Expr::Config(config_expr) =
                                            &item_schema_expr.config.node
                                        {
                                            if i < index {
                                                let mut items = config_expr.items.clone();
                                                items.append(&mut schema_config.items);
                                                schema_config.items = items;
                                            } else if i > index {
                                                let mut items = config_expr.items.clone();
                                                schema_config.items.append(&mut items);
                                            }
                                        }
                                    }
                                }
                            }
                            ast::Expr::Config(item_config_expr) => {
                                if let ast::Expr::Schema(schema_expr) =
                                    &mut merged_schema.node.value.node
                                {
                                    if let ast::Expr::Config(schema_config) =
                                        &mut schema_expr.config.node
                                    {
                                        if i < index {
                                            let mut items = item_config_expr.items.clone();
                                            items.append(&mut schema_config.items);
                                            schema_config.items = items;
                                        } else if i > index {
                                            let mut items = item_config_expr.items.clone();
                                            schema_config.items.append(&mut items);
                                        }
                                    }
                                }
                            }
                            _ => entries.push(item.clone()),
                        }
                    }
                    entries.push(Box::new(merged_schema));
                }
                None => entries.append(items),
            };
        }
    }
    // Unify config entries recursively.
    for entry in &mut entries {
        match &mut entry.node.value.node {
            ast::Expr::Schema(item_schema_expr) => {
                if let ast::Expr::Config(item_config_expr) = &mut item_schema_expr.config.node {
                    item_config_expr.items = unify_config_entries(&item_config_expr.items);
                }
            }
            ast::Expr::Config(item_config_expr) => {
                item_config_expr.items = unify_config_entries(&item_config_expr.items);
            }
            _ => {}
        }
    }
    entries
}

/// Merge program
pub fn merge_program(program: &mut ast::Program) {
    let mut merger = ConfigMergeTransformer {};
    merger.merge(program);
}

/// Fix AST config expr nest attribute declarations.
///
/// Examples
/// --------
/// {a.b.c = 1} -> {a: {b: {c = 1}}}
pub fn fix_config_expr_nest_attr(module: &mut ast::Module) {
    ConfigNestAttrTransformer::default().walk_module(module);
}
