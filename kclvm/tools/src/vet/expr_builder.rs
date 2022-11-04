use kclvm_ast::{
    ast::{
        ConfigEntry, ConfigEntryOperation, ConfigExpr, Expr, ExprContext, Identifier, ListExpr,
        NameConstant, NameConstantLit, Node, NodeRef, NumberLit, NumberLitValue, SchemaExpr,
        StringLit,
    },
    node_ref,
};

use crate::util::loader::{DataLoader, Loader, LoaderKind};
use anyhow::{bail, Context, Result};

trait ExprGenerator<T> {
    fn generate(&self, value: &T, schema_name: &Option<String>) -> Result<NodeRef<Expr>>;
}

/// `ExprBuilder` will generate ast expr from Json/Yaml.
/// `Object` in Json and `Mapping` in Yaml is mapped to `Schema Expr`.
/// You should set `schema_name` for `Schema Expr` before using `ExprBuilder`.
pub(crate) struct ExprBuilder {
    loader: DataLoader,
}

impl ExprBuilder {
    pub(crate) fn new_with_file_path(kind: LoaderKind, file_path: String) -> Result<Self> {
        let loader = DataLoader::new_with_file_path(kind, &file_path)
            .with_context(|| format!("Failed to Load '{}'", file_path))?;

        Ok(Self { loader })
    }

    #[allow(dead_code)]
    pub(crate) fn new_with_str(kind: LoaderKind, content: String) -> Result<Self> {
        let loader = DataLoader::new_with_str(kind, &content)
            .with_context(|| format!("Failed to Parse String '{}'", content))?;

        Ok(Self { loader })
    }

    /// Generate ast expr from Json/Yaml depends on `LoaderKind`.
    pub(crate) fn build(&self, schema_name: Option<String>) -> Result<NodeRef<Expr>> {
        match self.loader.get_kind() {
            LoaderKind::JSON => {
                let value = <DataLoader as Loader<serde_json::Value>>::load(&self.loader)
                    .with_context(|| format!("Failed to Load JSON"))?;
                Ok(self
                    .generate(&value, &schema_name)
                    .with_context(|| format!("Failed to Load JSON"))?)
            }
            LoaderKind::YAML => {
                let value = <DataLoader as Loader<serde_yaml::Value>>::load(&self.loader)
                    .with_context(|| format!("Failed to Load YAML"))?;
                Ok(self
                    .generate(&value, &schema_name)
                    .with_context(|| format!("Failed to Load YAML"))?)
            }
        }
    }
}

impl ExprGenerator<serde_yaml::Value> for ExprBuilder {
    fn generate(
        &self,
        value: &serde_yaml::Value,
        schema_name: &Option<String>,
    ) -> Result<NodeRef<Expr>> {
        match value {
            serde_yaml::Value::Null => Ok(node_ref!(Expr::NameConstantLit(NameConstantLit {
                value: NameConstant::None,
            }))),
            serde_yaml::Value::Bool(j_bool) => {
                let name_const = match NameConstant::try_from(*j_bool) {
                    Ok(nc) => nc,
                    Err(_) => {
                        bail!("Failed to Load Validated File")
                    }
                };

                Ok(node_ref!(Expr::NameConstantLit(NameConstantLit {
                    value: name_const
                })))
            }
            serde_yaml::Value::Number(j_num) => {
                if j_num.is_f64() {
                    let number_lit = match j_num.as_f64() {
                        Some(num_f64) => num_f64,
                        None => {
                            bail!("Failed to Load Validated File")
                        }
                    };

                    Ok(node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Float(number_lit)
                    })))
                } else if j_num.is_i64() {
                    let number_lit = match j_num.as_i64() {
                        Some(j_num) => j_num,
                        None => {
                            bail!("Failed to Load Validated File")
                        }
                    };

                    Ok(node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(number_lit)
                    })))
                } else {
                    bail!("Failed to Load Validated File, Unsupported Unsigned 64");
                }
            }
            serde_yaml::Value::String(j_string) => {
                let str_lit = match StringLit::try_from(j_string.to_string()) {
                    Ok(s) => s,
                    Err(_) => {
                        bail!("Failed to Load Validated File")
                    }
                };
                Ok(node_ref!(Expr::StringLit(str_lit)))
            }
            serde_yaml::Value::Sequence(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(
                        self.generate(j_arr_item, schema_name)
                            .with_context(|| format!("Failed to Load Validated File"))?,
                    );
                }
                Ok(node_ref!(Expr::List(ListExpr {
                    ctx: ExprContext::Load,
                    elts: j_arr_ast_nodes
                })))
            }
            serde_yaml::Value::Mapping(j_map) => {
                let mut config_entries: Vec<NodeRef<ConfigEntry>> = Vec::new();

                for (k, v) in j_map.iter() {
                    // The configuration builder already in the schema no longer needs a schema name
                    let k = self
                        .generate(k, &None)
                        .with_context(|| format!("Failed to Load Validated File"))?;
                    let v = self
                        .generate(v, &None)
                        .with_context(|| format!("Failed to Load Validated File"))?;

                    let config_entry = node_ref!(ConfigEntry {
                        key: Some(k),
                        value: v,
                        operation: ConfigEntryOperation::Union,
                        insert_index: -1
                    });
                    config_entries.push(config_entry);
                }

                let config_expr = node_ref!(Expr::Config(ConfigExpr {
                    items: config_entries
                }));

                match schema_name {
                    Some(s_name) => {
                        let iden = node_ref!(Identifier {
                            names: vec![s_name.to_string()],
                            pkgpath: String::new(),
                            ctx: ExprContext::Load
                        });
                        Ok(node_ref!(Expr::Schema(SchemaExpr {
                            name: iden,
                            config: config_expr,
                            args: vec![],
                            kwargs: vec![]
                        })))
                    }
                    None => Ok(config_expr),
                }
            }
            serde_yaml::Value::Tagged(_) => {
                bail!("Failed to Load Validated File, Unsupported Yaml Tagged.")
            }
        }
    }
}

impl ExprGenerator<serde_json::Value> for ExprBuilder {
    fn generate(
        &self,
        value: &serde_json::Value,
        schema_name: &Option<String>,
    ) -> Result<NodeRef<Expr>> {
        match value {
            serde_json::Value::Null => Ok(node_ref!(Expr::NameConstantLit(NameConstantLit {
                value: NameConstant::None,
            }))),
            serde_json::Value::Bool(j_bool) => {
                let name_const = match NameConstant::try_from(*j_bool) {
                    Ok(nc) => nc,
                    Err(_) => {
                        bail!("Failed to Load Validated File")
                    }
                };

                Ok(node_ref!(Expr::NameConstantLit(NameConstantLit {
                    value: name_const
                })))
            }
            serde_json::Value::Number(j_num) => {
                if j_num.is_f64() {
                    let number_lit = match j_num.as_f64() {
                        Some(num_f64) => num_f64,
                        None => {
                            bail!("Failed to Load Validated File")
                        }
                    };

                    Ok(node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Float(number_lit)
                    })))
                } else if j_num.is_i64() {
                    let number_lit = match j_num.as_i64() {
                        Some(j_num) => j_num,
                        None => {
                            bail!("Failed to Load Validated File")
                        }
                    };

                    Ok(node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(number_lit)
                    })))
                } else {
                    bail!("Failed to Load Validated File, Unsupported Unsigned 64");
                }
            }
            serde_json::Value::String(j_string) => {
                let str_lit = match StringLit::try_from(j_string.to_string()) {
                    Ok(s) => s,
                    Err(_) => {
                        bail!("Failed to Load Validated File")
                    }
                };

                Ok(node_ref!(Expr::StringLit(str_lit)))
            }
            serde_json::Value::Array(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(
                        self.generate(j_arr_item, schema_name)
                            .with_context(|| format!("Failed to Load Validated File"))?,
                    );
                }
                Ok(node_ref!(Expr::List(ListExpr {
                    ctx: ExprContext::Load,
                    elts: j_arr_ast_nodes
                })))
            }
            serde_json::Value::Object(j_map) => {
                let mut config_entries: Vec<NodeRef<ConfigEntry>> = Vec::new();

                for (k, v) in j_map.iter() {
                    let k = match StringLit::try_from(k.to_string()) {
                        Ok(s) => s,
                        Err(_) => {
                            bail!("Failed to Load Validated File")
                        }
                    };
                    let v = self
                        .generate(v, &None)
                        .with_context(|| format!("Failed to Load Validated File"))?;

                    let config_entry = node_ref!(ConfigEntry {
                        key: Some(node_ref!(Expr::StringLit(k))),
                        value: v,
                        operation: ConfigEntryOperation::Union,
                        insert_index: -1
                    });
                    config_entries.push(config_entry);
                }

                let config_expr = node_ref!(Expr::Config(ConfigExpr {
                    items: config_entries
                }));

                match schema_name {
                    Some(s_name) => {
                        let iden = node_ref!(Identifier {
                            names: vec![s_name.to_string()],
                            pkgpath: String::new(),
                            ctx: ExprContext::Load
                        });
                        Ok(node_ref!(Expr::Schema(SchemaExpr {
                            name: iden,
                            config: config_expr,
                            args: vec![],
                            kwargs: vec![]
                        })))
                    }
                    None => Ok(config_expr),
                }
            }
        }
    }
}
