use kclvm_ast::{
    ast::{
        ConfigEntry, ConfigEntryOperation, ConfigExpr, Expr, ExprContext, Identifier, ListExpr,
        NameConstant, NameConstantLit, Node, NodeRef, NumberLit, NumberLitValue, SchemaExpr,
        StringLit,
    },
    node_ref,
};

use crate::util::loader::{DataLoader, Loader, LoaderKind};
use anyhow::{Context, Result};

trait ExprGenerator<T> {
    fn generate(&self, value: &T) -> NodeRef<Expr>;
}

/// `ExprBuilder` will generate ast expr from Json/Yaml.
/// `Object` in Json and `Mapping` in Yaml is mapped to `Schema Expr`.
/// You should set `schema_name` for `Schema Expr` before using `ExprBuilder`.
pub(crate) struct ExprBuilder {
    schema_name: Option<String>,
    loader: DataLoader,
}

impl ExprBuilder {
    pub(crate) fn new_with_file_path(
        schema_name: Option<String>,
        kind: LoaderKind,
        file_path: String,
    ) -> Result<Self> {
        let loader = DataLoader::new_with_file_path(kind, &file_path)
            .with_context(|| format!("Failed to Load '{}'", file_path))?;

        Ok(Self {
            schema_name,
            loader,
        })
    }

    pub(crate) fn new_with_str(
        schema_name: Option<String>,
        kind: LoaderKind,
        content: String,
    ) -> Result<Self> {
        let loader = DataLoader::new_with_str(kind, &content)
            .with_context(|| format!("Failed to Parse String '{}'", content))?;

        Ok(Self {
            schema_name,
            loader,
        })
    }

    /// Generate ast expr from Json/Yaml depends on `LoaderKind`.
    pub(crate) fn build(&self) -> Result<NodeRef<Expr>> {
        match self.loader.get_kind() {
            LoaderKind::JSON => {
                let value = <DataLoader as Loader<serde_json::Value>>::load(&self.loader)
                    .with_context(|| format!("Failed to Load JSON"))?;
                Ok(self.generate(&value))
            }
            LoaderKind::YAML => {
                let value = <DataLoader as Loader<serde_yaml::Value>>::load(&self.loader)
                    .with_context(|| format!("Failed to Load YAML"))?;
                Ok(self.generate(&value))
            }
        }
    }
}

impl ExprGenerator<serde_yaml::Value> for ExprBuilder {
    fn generate(&self, value: &serde_yaml::Value) -> NodeRef<Expr> {
        match value {
            serde_yaml::Value::Null => {
                node_ref!(Expr::NameConstantLit(NameConstantLit {
                    value: NameConstant::None,
                }))
            }
            serde_yaml::Value::Bool(j_bool) => {
                node_ref!(Expr::NameConstantLit(NameConstantLit {
                    value: NameConstant::try_from(*j_bool).unwrap()
                }))
            }
            serde_yaml::Value::Number(j_num) => {
                if j_num.is_f64() {
                    node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Float(j_num.as_f64().unwrap())
                    }))
                } else if j_num.is_i64() {
                    node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(j_num.as_i64().unwrap())
                    }))
                } else {
                    node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(j_num.as_u64().unwrap().try_into().unwrap())
                    }))
                }
            }
            serde_yaml::Value::String(j_string) => {
                node_ref!(Expr::StringLit(
                    StringLit::try_from(j_string.to_string()).unwrap()
                ))
            }
            serde_yaml::Value::Sequence(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(self.generate(j_arr_item));
                }
                node_ref!(Expr::List(ListExpr {
                    ctx: ExprContext::Load,
                    elts: j_arr_ast_nodes
                }))
            }
            serde_yaml::Value::Mapping(j_map) => {
                let mut config_entries: Vec<NodeRef<ConfigEntry>> = Vec::new();

                for (k, v) in j_map.iter() {
                    let config_entry = node_ref!(ConfigEntry {
                        key: Some(self.generate(k)),
                        value: self.generate(v),
                        operation: ConfigEntryOperation::Union,
                        insert_index: -1
                    });
                    config_entries.push(config_entry);
                }

                let config_expr = node_ref!(Expr::Config(ConfigExpr {
                    items: config_entries
                }));

                match &self.schema_name {
                    Some(s_name) => {
                        let iden = node_ref!(Identifier {
                            names: vec![s_name.to_string()],
                            pkgpath: String::new(),
                            ctx: ExprContext::Load
                        });
                        node_ref!(Expr::Schema(SchemaExpr {
                            name: iden,
                            config: config_expr,
                            args: vec![],
                            kwargs: vec![]
                        }))
                    }
                    None => config_expr,
                }
            }
            serde_yaml::Value::Tagged(_) => {
                bug!("Yaml Tagged is not supported in KCL_Vet, this is an internal bug.")
            }
        }
    }
}

impl ExprGenerator<serde_json::Value> for ExprBuilder {
    fn generate(&self, value: &serde_json::Value) -> NodeRef<Expr> {
        match value {
            serde_json::Value::Null => {
                node_ref!(Expr::NameConstantLit(NameConstantLit {
                    value: NameConstant::None,
                }))
            }
            serde_json::Value::Bool(j_bool) => {
                node_ref!(Expr::NameConstantLit(NameConstantLit {
                    value: NameConstant::try_from(*j_bool).unwrap()
                }))
            }
            serde_json::Value::Number(j_num) => {
                if j_num.is_f64() {
                    node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Float(j_num.as_f64().unwrap())
                    }))
                } else if j_num.is_i64() {
                    node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(j_num.as_i64().unwrap())
                    }))
                } else {
                    node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(j_num.as_u64().unwrap().try_into().unwrap())
                    }))
                }
            }
            serde_json::Value::String(j_string) => {
                node_ref!(Expr::StringLit(
                    StringLit::try_from(j_string.to_string()).unwrap()
                ))
            }
            serde_json::Value::Array(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(self.generate(j_arr_item));
                }
                node_ref!(Expr::List(ListExpr {
                    ctx: ExprContext::Load,
                    elts: j_arr_ast_nodes
                }))
            }
            serde_json::Value::Object(j_map) => {
                let mut config_entries: Vec<NodeRef<ConfigEntry>> = Vec::new();

                for (k, v) in j_map.iter() {
                    let config_entry = node_ref!(ConfigEntry {
                        key: Some(node_ref!(Expr::StringLit(
                            StringLit::try_from(k.to_string()).unwrap()
                        ))),
                        value: self.generate(v),
                        operation: ConfigEntryOperation::Union,
                        insert_index: -1
                    });
                    config_entries.push(config_entry);
                }

                let config_expr = node_ref!(Expr::Config(ConfigExpr {
                    items: config_entries
                }));

                match &self.schema_name {
                    Some(s_name) => {
                        let iden = node_ref!(Identifier {
                            names: vec![s_name.to_string()],
                            pkgpath: String::new(),
                            ctx: ExprContext::Load
                        });
                        node_ref!(Expr::Schema(SchemaExpr {
                            name: iden,
                            config: config_expr,
                            args: vec![],
                            kwargs: vec![]
                        }))
                    }
                    None => config_expr,
                }
            }
        }
    }
}
