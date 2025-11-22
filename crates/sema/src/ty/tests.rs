use super::*;

#[test]
fn test_sup() {
    let cases = vec![
        (vec![], Arc::new(Type::ANY)),
        (vec![Arc::new(Type::ANY)], Arc::new(Type::ANY)),
        (vec![Arc::new(Type::STR)], Arc::new(Type::STR)),
        (
            vec![Arc::new(Type::STR), Arc::new(Type::INT)],
            Type::union_ref(&[Arc::new(Type::STR), Arc::new(Type::INT)]),
        ),
        (
            vec![Arc::new(Type::BOOL), Arc::new(Type::bool_lit(true))],
            Arc::new(Type::BOOL),
        ),
        (
            vec![
                Arc::new(Type::str_lit("Blue")),
                Arc::new(Type::str_lit("Yellow")),
                Arc::new(Type::str_lit("Red")),
            ],
            Type::union_ref(&[
                Arc::new(Type::str_lit("Blue")),
                Arc::new(Type::str_lit("Yellow")),
                Arc::new(Type::str_lit("Red")),
            ]),
        ),
        (
            vec![
                Type::list_ref(Type::union_ref(&[
                    Arc::new(Type::int_lit(1)),
                    Arc::new(Type::int_lit(2)),
                ])),
                Type::list_ref(Type::union_ref(&[
                    Arc::new(Type::int_lit(3)),
                    Arc::new(Type::int_lit(4)),
                ])),
            ],
            Type::union_ref(&[
                Type::list_ref(Type::union_ref(&[
                    Arc::new(Type::int_lit(1)),
                    Arc::new(Type::int_lit(2)),
                ])),
                Type::list_ref(Type::union_ref(&[
                    Arc::new(Type::int_lit(3)),
                    Arc::new(Type::int_lit(4)),
                ])),
            ]),
        ),
        (
            vec![
                Type::union_ref(&[
                    Arc::new(Type::STR),
                    Type::dict_ref(Arc::new(Type::STR), Arc::new(Type::STR)),
                ]),
                Type::dict_ref(Arc::new(Type::ANY), Arc::new(Type::ANY)),
            ],
            Type::union_ref(&[
                Arc::new(Type::STR),
                Type::dict_ref(Arc::new(Type::ANY), Arc::new(Type::ANY)),
            ]),
        ),
    ];
    for (types, expected) in &cases {
        let got = sup(types);
        assert_eq!(got, *expected);
    }
}

#[test]
fn test_type_walker() {
    fn walk_fn(ty: &Type) -> TypeRef {
        if ty.is_int() {
            Arc::new(Type::STR)
        } else {
            Arc::new(ty.clone())
        }
    }
    let cases = [
        (Arc::new(Type::ANY), Arc::new(Type::ANY)),
        (Arc::new(Type::INT), Arc::new(Type::STR)),
        (Arc::new(Type::STR), Arc::new(Type::STR)),
        (
            Type::list_ref(Arc::new(Type::INT)),
            Type::list_ref(Arc::new(Type::STR)),
        ),
        (
            Type::union_ref(&[Arc::new(Type::INT), Arc::new(Type::STR)]),
            Type::union_ref(&[Arc::new(Type::STR), Arc::new(Type::STR)]),
        ),
        (
            Type::union_ref(&[
                Arc::new(Type::INT),
                Arc::new(Type::STR),
                Type::union_ref(&[Arc::new(Type::INT), Arc::new(Type::STR)]),
            ]),
            Type::union_ref(&[
                Arc::new(Type::STR),
                Arc::new(Type::STR),
                Type::union_ref(&[Arc::new(Type::STR), Arc::new(Type::STR)]),
            ]),
        ),
        (
            Type::dict_ref(Arc::new(Type::INT), Arc::new(Type::INT)),
            Type::dict_ref(Arc::new(Type::STR), Arc::new(Type::STR)),
        ),
    ];
    for (ty, expected) in cases {
        assert_eq!(
            walker::walk_type(&ty, walk_fn),
            expected,
            "Type test failed: {}",
            ty.ty_str()
        );
    }
}
