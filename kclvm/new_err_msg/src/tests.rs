use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};

use crate::fluent::{self, typeck};

#[test]
fn test_locale_message() {
    let langid_en = "en-US".parse().expect("Parsing failed");
    let mut bundle: FluentBundle<FluentResource> = FluentBundle::new(vec![langid_en]);
    for locale_res in fluent::DEFAULT_LOCALE_RESOURCES {
        let resource = locale_res.to_string();
        println!("resource - {}", resource);

        let res =
            FluentResource::try_new(resource.to_string()).expect("Failed to parse an FTL string.");
        bundle
            .add_resource(res)
            .expect("Failed to add FTL resources to the bundle.");
    }
    let msg = bundle
        .get_message("typeck-field-multiply-specified-in-initializer")
        .expect("Message doesn't exist.");
    let mut errors = vec![];
    let pattern = msg.value().expect("Message has no value.");
    let value = bundle.format_pattern(&pattern, None, &mut errors);

    assert_eq!(
        &value,
        "field `\u{2068}{$ident}\u{2069}` specified more than once"
    );

    let mut args = FluentArgs::new();
    args.set("ident", FluentValue::from("John"));

    let msg = bundle
        .get_message("typeck-field-multiply-specified-in-initializer")
        .expect("Message doesn't exist.");
    let mut errors = vec![];
    let pattern = msg.value().expect("Message has no value.");
    let value = bundle.format_pattern(&pattern, Some(&args), &mut errors);
    assert_eq!(
        &value,
        "field `\u{2068}John\u{2069}` specified more than once"
    );

    let label_attr = msg.get_attribute("label").expect("Attr doesn't exist.");
    let attr_pattern = label_attr.value();
    let value = bundle.format_pattern(&attr_pattern, None, &mut errors);
    assert_eq!(&value, "used more than once");
}
