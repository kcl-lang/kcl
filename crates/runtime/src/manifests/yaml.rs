use crate::{Context, ValueRef, YamlEncodeOptions};

/// Encode the list value to the yaml stream format.
#[inline]
pub(crate) fn encode_yaml_stream_to_manifests(
    ctx: &mut Context,
    values: &ValueRef,
    opts: YamlEncodeOptions,
) {
    // Update custom plan manifests output.
    ctx.buffer.custom_manifests_output = Some(
        values
            .as_list_ref()
            .values
            .iter()
            .map(|v| {
                let s = v.to_yaml_string_with_options(&opts);
                s.strip_suffix('\n').unwrap_or(&s).to_string()
            })
            .collect::<Vec<String>>()
            .join(&format!("\n{}\n", opts.sep)),
    );
    // Update plan options.
    ctx.plan_opts.disable_none = opts.ignore_none;
    ctx.plan_opts.sort_keys = opts.sort_keys;
    ctx.plan_opts.show_hidden = !opts.ignore_private;
    ctx.plan_opts.sep = Some(opts.sep.clone())
}

#[cfg(test)]
mod test_manifests_yaml {
    use crate::{manifests::yaml::encode_yaml_stream_to_manifests, *};

    #[test]
    fn test_encode_yaml_stream_to_manifests() {
        let cases = [
            (
                "a: 1",
                ValueRef::list(Some(&[&ValueRef::dict(Some(&[("a", &ValueRef::int(1))]))])),
                YamlEncodeOptions::default(),
            ),
            (
                "a: 1\nb: 2",
                ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                    ("a", &ValueRef::int(1)),
                    ("b", &ValueRef::int(2)),
                ]))])),
                YamlEncodeOptions::default(),
            ),
            (
                "a:\n- 1\n- 2\n- 3\nb: s",
                ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                    ("a", &ValueRef::list_int(&[1, 2, 3])),
                    ("b", &ValueRef::str("s")),
                ]))])),
                YamlEncodeOptions::default(),
            ),
            (
                "a: 1",
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
                "a: 1\nb: null",
                ValueRef::list(Some(&[&ValueRef::dict(Some(&[
                    ("a", &ValueRef::int(1)),
                    ("b", &ValueRef::none()),
                ]))])),
                YamlEncodeOptions::default(),
            ),
            (
                "a: 1",
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
        let mut ctx = Context::default();
        for (yaml_str, value, opts) in cases {
            encode_yaml_stream_to_manifests(&mut ctx, &value, opts);
            assert_eq!(
                Some(yaml_str.to_string()),
                ctx.buffer.custom_manifests_output
            );
        }
    }

    #[test]
    fn test_encode_yaml_stream_to_manifests_failure() {
        let prev_hook = std::panic::take_hook();
        // Disable print panic info in stderr.
        std::panic::set_hook(Box::new(|_| {}));
        assert_panic("invalid list value", || {
            let mut ctx = Context::default();
            let value = ValueRef::dict(Some(&[("a", &ValueRef::int(1))]));
            let opts = YamlEncodeOptions::default();
            encode_yaml_stream_to_manifests(&mut ctx, &value, opts);
        });
        std::panic::set_hook(prev_hook);
    }
}
