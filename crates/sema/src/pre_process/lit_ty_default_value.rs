use kcl_ast::ast;
use kcl_ast::walker::MutSelfMutWalker;

#[derive(Default)]
struct LitTypeDefaultValueTransformer;

impl<'ctx> MutSelfMutWalker<'ctx> for LitTypeDefaultValueTransformer {
    fn walk_schema_attr(&mut self, schema_attr: &'ctx mut ast::SchemaAttr) {
        if schema_attr.value.is_none()
            && !schema_attr.is_optional
            && let ast::Type::Literal(literal_ty) = &schema_attr.ty.node
        {
            let filename = schema_attr.ty.filename.clone();
            let line = schema_attr.ty.end_line;
            // Append ` = ` width for th column.
            let column = schema_attr.ty.end_column + 3;
            schema_attr.op = Some(ast::AugOp::Assign);
            match literal_ty {
                ast::LiteralType::Bool(val) => {
                    let column_offset = if *val { 4 } else { 5 };
                    schema_attr.value = Some(Box::new(ast::Node::new(
                        ast::Expr::NameConstantLit(ast::NameConstantLit {
                            value: if *val {
                                ast::NameConstant::True
                            } else {
                                ast::NameConstant::False
                            },
                        }),
                        filename,
                        line,
                        column,
                        line,
                        column + column_offset,
                    )));
                }
                ast::LiteralType::Int(val) => {
                    let value = val.value.to_string();
                    let mut column_offset = value.len() as u64;
                    if let Some(suffix) = &val.suffix {
                        column_offset += suffix.value().len() as u64
                    }
                    schema_attr.value = Some(Box::new(ast::Node::new(
                        ast::Expr::NumberLit(ast::NumberLit {
                            binary_suffix: val.suffix.clone(),
                            value: ast::NumberLitValue::Int(val.value),
                        }),
                        filename,
                        line,
                        column,
                        line,
                        column + column_offset,
                    )));
                }
                ast::LiteralType::Float(val) => {
                    let value = kcl_runtime::float_to_string(*val);
                    let column_offset = value.len() as u64;
                    schema_attr.value = Some(Box::new(ast::Node::new(
                        ast::Expr::NumberLit(ast::NumberLit {
                            binary_suffix: None,
                            value: ast::NumberLitValue::Float(*val),
                        }),
                        filename,
                        line,
                        column,
                        line,
                        column + column_offset,
                    )));
                }
                ast::LiteralType::Str(val) => {
                    let value: ast::StringLit = val.to_string().into();
                    let column_offset = value.raw_value.len() as u64;
                    schema_attr.value = Some(Box::new(ast::Node::new(
                        ast::Expr::StringLit(value),
                        filename,
                        line,
                        column,
                        line,
                        column + column_offset,
                    )));
                }
            }
        }
    }
}

/// Fix literal type default value. e.g., `a: "value"` -> `a: "value" = "value"`.
#[inline]
pub fn fix_lit_ty_default_value(module: &'_ mut ast::Module) {
    LitTypeDefaultValueTransformer.walk_module(module);
}
