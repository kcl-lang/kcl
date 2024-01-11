use compiler_base_span::span::new_byte_pos;
use kclvm_ast::{
    ast::{
        ConfigEntry, ConfigEntryOperation, ConfigExpr, Expr, ExprContext, Identifier, ListExpr,
        NameConstant, NameConstantLit, Node, NodeRef, NumberLit, NumberLitValue, SchemaExpr,
        StringLit,
    },
    node_ref,
};
use serde_json::json;

use crate::util::loader::{DataLoader, Loader, LoaderKind};
use anyhow::{bail, Context, Result};

const FAIL_LOAD_VALIDATED_ERR_MSG: &str = "Failed to load the validated file";

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
                let value = <DataLoader as Loader<
                    json_spanned_value::Spanned<json_spanned_value::Value>,
                >>::load(&self.loader)
                .with_context(|| "Failed to Load JSON".to_string())?;
                Ok(self
                    .generate(&value, &schema_name)
                    .with_context(|| "Failed to Load JSON".to_string())?)
            }
            LoaderKind::YAML => {
                let value = <DataLoader as Loader<located_yaml::Yaml>>::load(&self.loader)
                    .with_context(|| "Failed to Load YAML".to_string())?;
                Ok(self
                    .generate(&value, &schema_name)
                    .with_context(|| "Failed to Load YAML".to_string())?)
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
                    Err(err) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, {err}")
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
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
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
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                        }
                    };

                    Ok(node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(number_lit)
                    })))
                } else {
                    bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Unsigned 64");
                }
            }
            serde_yaml::Value::String(j_string) => {
                let str_lit = match StringLit::try_from(j_string.to_string()) {
                    Ok(s) => s,
                    Err(_) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                    }
                };
                Ok(node_ref!(Expr::StringLit(str_lit)))
            }
            serde_yaml::Value::Sequence(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(
                        self.generate(j_arr_item, schema_name)
                            .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?,
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
                        .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?;
                    let v = self
                        .generate(v, &None)
                        .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?;

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
                            names: vec![Node::dummy_node(s_name.to_string())],
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
            serde_yaml::Value::Tagged(v) => {
                bail!(
                    "{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Yaml tag {}",
                    v.tag
                )
            }
        }
    }
}

impl ExprGenerator<located_yaml::Yaml> for ExprBuilder {
    fn generate(
        &self,
        value: &located_yaml::Yaml,
        schema_name: &Option<String>,
    ) -> Result<NodeRef<Expr>> {
        let loc = (
            self.loader.file_name(),
            value.marker.line as u64,
            value.marker.col as u64,
            0,
            0,
        );
        match &value.yaml {
            located_yaml::YamlElt::Null => Ok(node_ref!(
                Expr::NameConstantLit(NameConstantLit {
                    value: NameConstant::None,
                }),
                loc
            )),
            located_yaml::YamlElt::Boolean(j_bool) => {
                let name_const = match NameConstant::try_from(*j_bool) {
                    Ok(nc) => nc,
                    Err(err) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, {err}")
                    }
                };

                Ok(node_ref!(
                    Expr::NameConstantLit(NameConstantLit { value: name_const }),
                    loc
                ))
            }
            located_yaml::YamlElt::Integer(j_int) => {
                if json!(j_int).is_i64() {
                    Ok(node_ref!(
                        Expr::NumberLit(NumberLit {
                            binary_suffix: None,
                            value: NumberLitValue::Int(*j_int)
                        }),
                        loc
                    ))
                } else {
                    bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Number Type");
                }
            }
            located_yaml::YamlElt::Real(j_float) => {
                if let Ok(number_lit) = j_float.parse::<f64>() {
                    if format!("{}", number_lit) != *j_float {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Number Type",)
                    }
                    Ok(node_ref!(
                        Expr::NumberLit(NumberLit {
                            binary_suffix: None,
                            value: NumberLitValue::Float(number_lit)
                        }),
                        loc
                    ))
                } else {
                    bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Number Type",)
                }
            }
            located_yaml::YamlElt::String(j_string) => {
                let str_lit = match StringLit::try_from(j_string.to_string()) {
                    Ok(s) => s,
                    Err(_) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                    }
                };
                Ok(node_ref!(Expr::StringLit(str_lit), loc))
            }
            located_yaml::YamlElt::Array(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(
                        self.generate(j_arr_item, schema_name)
                            .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?,
                    );
                }
                Ok(node_ref!(
                    Expr::List(ListExpr {
                        ctx: ExprContext::Load,
                        elts: j_arr_ast_nodes
                    }),
                    loc
                ))
            }
            located_yaml::YamlElt::Hash(j_map) => {
                let mut config_entries: Vec<NodeRef<ConfigEntry>> = Vec::new();

                for (k, v) in j_map.iter() {
                    // The configuration builder already in the schema no longer needs a schema name
                    let k = self
                        .generate(k, &None)
                        .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?;
                    let v = self
                        .generate(v, &None)
                        .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?;

                    let config_entry = node_ref!(
                        ConfigEntry {
                            key: Some(k),
                            value: v,
                            operation: ConfigEntryOperation::Union,
                            insert_index: -1
                        },
                        loc.clone()
                    );
                    config_entries.push(config_entry);
                }

                let config_expr = node_ref!(
                    Expr::Config(ConfigExpr {
                        items: config_entries
                    }),
                    loc.clone()
                );

                match schema_name {
                    Some(s_name) => {
                        let iden = node_ref!(
                            Identifier {
                                names: vec![Node::new(
                                    s_name.to_string(),
                                    loc.0.clone(),
                                    loc.1,
                                    loc.2,
                                    loc.3,
                                    loc.4
                                )],
                                pkgpath: String::new(),
                                ctx: ExprContext::Load
                            },
                            loc.clone()
                        );
                        Ok(node_ref!(
                            Expr::Schema(SchemaExpr {
                                name: iden,
                                config: config_expr,
                                args: vec![],
                                kwargs: vec![]
                            }),
                            loc.clone()
                        ))
                    }
                    None => Ok(config_expr),
                }
            }
            _ => {
                bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Yaml Element",)
            }
        }
    }
}

/// `ExprBuilder` will generate ast expr from Json with span.
impl ExprGenerator<json_spanned_value::Spanned<json_spanned_value::Value>> for ExprBuilder {
    fn generate(
        &self,
        value: &json_spanned_value::Spanned<json_spanned_value::Value>,
        schema_name: &Option<String>,
    ) -> Result<NodeRef<Expr>> {
        let loc = self.loader.byte_pos_to_pos_in_sourcemap(
            new_byte_pos(value.span().0 as u32),
            new_byte_pos(value.span().1 as u32),
        );
        match value.get_ref() {
            json_spanned_value::Value::Null => Ok(node_ref!(
                Expr::NameConstantLit(NameConstantLit {
                    value: NameConstant::None,
                }),
                loc
            )),
            json_spanned_value::Value::Bool(j_bool) => {
                let name_const = match NameConstant::try_from(*j_bool) {
                    Ok(nc) => nc,
                    Err(err) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, {err}")
                    }
                };

                Ok(node_ref!(
                    Expr::NameConstantLit(NameConstantLit { value: name_const }),
                    loc
                ))
            }
            json_spanned_value::Value::Number(j_num) => {
                if j_num.is_f64() {
                    let number_lit = match j_num.as_f64() {
                        Some(num_f64) => num_f64,
                        None => {
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                        }
                    };

                    Ok(node_ref!(
                        Expr::NumberLit(NumberLit {
                            binary_suffix: None,
                            value: NumberLitValue::Float(number_lit)
                        }),
                        loc
                    ))
                } else if j_num.is_i64() {
                    let number_lit = match j_num.as_i64() {
                        Some(j_num) => j_num,
                        None => {
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                        }
                    };

                    Ok(node_ref!(
                        Expr::NumberLit(NumberLit {
                            binary_suffix: None,
                            value: NumberLitValue::Int(number_lit)
                        }),
                        loc
                    ))
                } else {
                    bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Unsigned 64");
                }
            }
            json_spanned_value::Value::String(j_string) => {
                let str_lit = match StringLit::try_from(j_string.to_string()) {
                    Ok(s) => s,
                    Err(_) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                    }
                };

                Ok(node_ref!(Expr::StringLit(str_lit), loc))
            }
            json_spanned_value::Value::Array(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(
                        self.generate(j_arr_item, schema_name)
                            .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?,
                    );
                }
                Ok(node_ref!(
                    Expr::List(ListExpr {
                        ctx: ExprContext::Load,
                        elts: j_arr_ast_nodes
                    }),
                    loc
                ))
            }
            json_spanned_value::Value::Object(j_map) => {
                let mut config_entries: Vec<NodeRef<ConfigEntry>> = Vec::new();

                for (k, v) in j_map.iter() {
                    let k_span = k.span();
                    let k = match StringLit::try_from(k.to_string()) {
                        Ok(s) => s,
                        Err(err) => {
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, {err}")
                        }
                    };
                    let v = self
                        .generate(v, &None)
                        .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?;

                    let config_entry = node_ref!(
                        ConfigEntry {
                            key: Some(node_ref!(
                                Expr::StringLit(k),
                                self.loader.byte_pos_to_pos_in_sourcemap(
                                    new_byte_pos(k_span.0 as u32),
                                    new_byte_pos(k_span.1 as u32)
                                )
                            )),
                            value: v,
                            operation: ConfigEntryOperation::Union,
                            insert_index: -1
                        },
                        loc.clone()
                    );
                    config_entries.push(config_entry);
                }

                let config_expr = node_ref!(
                    Expr::Config(ConfigExpr {
                        items: config_entries
                    }),
                    loc.clone()
                );

                match schema_name {
                    Some(s_name) => {
                        let iden = node_ref!(
                            Identifier {
                                names: vec![Node::new(
                                    s_name.to_string(),
                                    loc.0.clone(),
                                    loc.1,
                                    loc.2,
                                    loc.3,
                                    loc.4
                                )],
                                pkgpath: String::new(),
                                ctx: ExprContext::Load
                            },
                            loc.clone()
                        );
                        Ok(node_ref!(
                            Expr::Schema(SchemaExpr {
                                name: iden,
                                config: config_expr,
                                args: vec![],
                                kwargs: vec![]
                            }),
                            loc
                        ))
                    }
                    None => Ok(config_expr),
                }
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
                    Err(err) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, {err}")
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
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
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
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                        }
                    };

                    Ok(node_ref!(Expr::NumberLit(NumberLit {
                        binary_suffix: None,
                        value: NumberLitValue::Int(number_lit)
                    })))
                } else {
                    bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, Unsupported Unsigned 64");
                }
            }
            serde_json::Value::String(j_string) => {
                let str_lit = match StringLit::try_from(j_string.to_string()) {
                    Ok(s) => s,
                    Err(_) => {
                        bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}")
                    }
                };

                Ok(node_ref!(Expr::StringLit(str_lit)))
            }
            serde_json::Value::Array(j_arr) => {
                let mut j_arr_ast_nodes: Vec<NodeRef<Expr>> = Vec::new();
                for j_arr_item in j_arr {
                    j_arr_ast_nodes.push(
                        self.generate(j_arr_item, schema_name)
                            .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?,
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
                        Err(err) => {
                            bail!("{FAIL_LOAD_VALIDATED_ERR_MSG}, {err}")
                        }
                    };
                    let v = self
                        .generate(v, &None)
                        .with_context(|| FAIL_LOAD_VALIDATED_ERR_MSG)?;

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
                            names: vec![Node::dummy_node(s_name.to_string())],
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
