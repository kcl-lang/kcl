use prost_wkt_types::*;
use std::collections::HashMap;

fn create_struct() -> Value {
    let number: Value = Value::from(10.0);
    let null: Value = Value::null();
    let string: Value = Value::from(String::from("Hello"));
    let list = vec![Value::null(), Value::from(100.0)];
    let pb_list: Value = Value::from(list);
    let mut map: HashMap<String, Value> = HashMap::new();
    map.insert(String::from("number"), number);
    map.insert(String::from("null"), null);
    map.insert(String::from("some_string"), string);
    map.insert(String::from("list"), pb_list);
    Value::from(map)
}

#[test]
fn test_serde() {
    let value = create_struct();
    let string = serde_json::to_string_pretty(&value).expect("Json string");
    println!("{string}");
    let back: Value = serde_json::from_str(&string).expect("Value");
    println!("{back:?}");
    assert_eq!(value, back);
}

#[test]
fn test_flatten_struct() {
    let mut fields: HashMap<String, Value> = HashMap::new();
    fields.insert("test".to_string(), create_struct());
    let strct = Struct {
        fields: fields.clone(),
    };
    let string_strct = serde_json::to_string_pretty(&strct).expect("Serialized struct");
    println!("{string_strct}");

    let value = Value::from(fields);
    let string = serde_json::to_string_pretty(&value).expect("A Value serialized to string");
    println!("{string}");

    assert_eq!(string_strct, string);
}

#[test]
fn test_flatten_list() {
    let values: Vec<Value> = vec![Value::null(), Value::from(20.0), Value::from(true)];
    let list: ListValue = ListValue {
        values: values.clone(),
    };
    let string_list = serde_json::to_string_pretty(&list).expect("Serialized list");
    println!("{string_list}");

    let value = Value::from(values);
    let string = serde_json::to_string_pretty(&value).expect("A Value serialized to string");
    println!("{string}");

    assert_eq!(string_list, string);
}
