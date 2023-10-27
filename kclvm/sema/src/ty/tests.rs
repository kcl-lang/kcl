use super::*;

#[test]
fn test_sup() {
    let cases = vec![
        (vec![], Rc::new(Type::ANY)),
        (vec![Rc::new(Type::ANY)], Rc::new(Type::ANY)),
        (vec![Rc::new(Type::STR)], Rc::new(Type::STR)),
        (
            vec![Rc::new(Type::STR), Rc::new(Type::INT)],
            Type::union_ref(&[Rc::new(Type::STR), Rc::new(Type::INT)]),
        ),
        (
            vec![Rc::new(Type::BOOL), Rc::new(Type::bool_lit(true))],
            Rc::new(Type::BOOL),
        ),
        (
            vec![
                Rc::new(Type::str_lit("Blue")),
                Rc::new(Type::str_lit("Yellow")),
                Rc::new(Type::str_lit("Red")),
            ],
            Type::union_ref(&[
                Rc::new(Type::str_lit("Blue")),
                Rc::new(Type::str_lit("Yellow")),
                Rc::new(Type::str_lit("Red")),
            ]),
        ),
        (
            vec![
                Type::list_ref(Type::union_ref(&[
                    Rc::new(Type::int_lit(1)),
                    Rc::new(Type::int_lit(2)),
                ])),
                Type::list_ref(Type::union_ref(&[
                    Rc::new(Type::int_lit(3)),
                    Rc::new(Type::int_lit(4)),
                ])),
            ],
            Type::union_ref(&[
                Type::list_ref(Type::union_ref(&[
                    Rc::new(Type::int_lit(1)),
                    Rc::new(Type::int_lit(2)),
                ])),
                Type::list_ref(Type::union_ref(&[
                    Rc::new(Type::int_lit(3)),
                    Rc::new(Type::int_lit(4)),
                ])),
            ]),
        ),
        (
            vec![
                Type::union_ref(&[
                    Rc::new(Type::STR),
                    Type::dict_ref(Rc::new(Type::STR), Rc::new(Type::STR)),
                ]),
                Type::dict_ref(Rc::new(Type::ANY), Rc::new(Type::ANY)),
            ],
            Type::union_ref(&[
                Rc::new(Type::STR),
                Type::dict_ref(Rc::new(Type::ANY), Rc::new(Type::ANY)),
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
            Rc::new(Type::STR)
        } else {
            Rc::new(ty.clone())
        }
    }
    let cases = [
        (Rc::new(Type::ANY), Rc::new(Type::ANY)),
        (Rc::new(Type::INT), Rc::new(Type::STR)),
        (Rc::new(Type::STR), Rc::new(Type::STR)),
        (
            Type::list_ref(Rc::new(Type::INT)),
            Type::list_ref(Rc::new(Type::STR)),
        ),
        (
            Type::union_ref(&[Rc::new(Type::INT), Rc::new(Type::STR)]),
            Type::union_ref(&[Rc::new(Type::STR), Rc::new(Type::STR)]),
        ),
        (
            Type::union_ref(&[
                Rc::new(Type::INT),
                Rc::new(Type::STR),
                Type::union_ref(&[Rc::new(Type::INT), Rc::new(Type::STR)]),
            ]),
            Type::union_ref(&[
                Rc::new(Type::STR),
                Rc::new(Type::STR),
                Type::union_ref(&[Rc::new(Type::STR), Rc::new(Type::STR)]),
            ]),
        ),
        (
            Type::dict_ref(Rc::new(Type::INT), Rc::new(Type::INT)),
            Type::dict_ref(Rc::new(Type::STR), Rc::new(Type::STR)),
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
