use crate::*;

#[test]
fn test_kclvm_manifests_yaml_stream() {
    let cases = [
        (
            "a: 1\n",
            ValueRef::list(Some(&[&ValueRef::dict(Some(&[("a", &ValueRef::int(1))]))])),
            YamlEncodeOptions::default(),
        ),
        (
            "a: 1\nb: 2\n",
            ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                ("a", &ValueRef::int(1)),
                ("b", &ValueRef::int(2)),
            ]))])),
            YamlEncodeOptions::default(),
        ),
        (
            "a:\n- 1\n- 2\n- 3\nb: s\n",
            ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                ("a", &ValueRef::list_int(&[1, 2, 3])),
                ("b", &ValueRef::str("s")),
            ]))])),
            YamlEncodeOptions::default(),
        ),
        (
            "a: 1\n",
            ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                ("a", &ValueRef::int(1)),
                ("_b", &ValueRef::none()),
            ]))])),
            YamlEncodeOptions {
                ignore_private: true,
                ..Default::default()
            },
        ),
        (
            "a: 1\nb: null\n",
            ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                ("a", &ValueRef::int(1)),
                ("b", &ValueRef::none()),
            ]))])),
            YamlEncodeOptions::default(),
        ),
        (
            "a: 1\n",
            ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                ("a", &ValueRef::int(1)),
                ("_b", &ValueRef::int(2)),
                ("c", &ValueRef::none()),
                ("d", &ValueRef::undefined()),
            ]))])),
            YamlEncodeOptions {
                ignore_private: true,
                ignore_none: true,
                ..Default::default()
            },
        ),
    ];
    for (yaml_str, value, opts) in cases {
        let mut ctx = Context::default();
        let opts = ValueRef::dict(Some(&[
            ("sort_keys", &ValueRef::bool(opts.sort_keys)),
            ("ignore_private", &ValueRef::bool(opts.ignore_private)),
            ("ignore_none", &ValueRef::bool(opts.ignore_none)),
            ("sep", &ValueRef::str(&opts.sep)),
        ]));
        let mut args = ValueRef::list(None);
        args.list_append(&value);
        let mut kwargs = ValueRef::dict(None);
        kwargs.dict_insert(
            &mut ctx,
            "opts",
            &opts,
            ConfigEntryOperationKind::Override,
            -1,
        );
        kclvm_manifests_yaml_stream(&mut ctx, &args, &kwargs);
        assert_eq!(
            Some(yaml_str.to_string()),
            ctx.buffer.custom_manifests_output
        );
    }
}

#[test]
fn test_kclvm_manifests_yaml_stream_invalid() {
    let prev_hook = std::panic::take_hook();
    // Disable print panic info in stderr.
    std::panic::set_hook(Box::new(|_| {}));
    assert_panic(
        "yaml_stream() missing 1 required positional argument: 'values'",
        || {
            let mut ctx = Context::new();
            let args = ValueRef::list(None).into_raw(&mut ctx);
            let kwargs = ValueRef::dict(None).into_raw(&mut ctx);
            kclvm_manifests_yaml_stream(ctx.into_raw(), args, kwargs);
        },
    );
    assert_panic(
        "Invalid options arguments in yaml_stream(): expect config, got str",
        || {
            let mut ctx = Context::new();
            let args = ValueRef::list(None).into_raw(&mut ctx);
            let kwargs = ValueRef::dict(Some(&[("opts", &ValueRef::str("invalid_kwarg"))]))
                .into_raw(&mut ctx);
            kclvm_manifests_yaml_stream(ctx.into_raw(), args, kwargs);
        },
    );
    assert_panic(
        "Invalid options arguments in yaml_stream(): expect config, got NoneType",
        || {
            let mut ctx = Context::new();
            let args = ValueRef::list(None).into_raw(&mut ctx);
            let kwargs = ValueRef::dict(Some(&[("opts", &ValueRef::none())])).into_raw(&mut ctx);
            kclvm_manifests_yaml_stream(ctx.into_raw(), args, kwargs);
        },
    );
    std::panic::set_hook(prev_hook);
}
