use protobuf::MessageFull;
use protobuf_json_mapping::PrintOptions;

//Parse byte sequence into protobuf message
pub fn parse_message_from_protobuf<M: MessageFull>(p: &[u8]) -> M {
    M::parse_from_bytes(p).unwrap()
}

//Parse json string into protobuf message
pub fn parse_message_from_json<M: MessageFull>(s: &str) -> M {
    protobuf_json_mapping::parse_from_str::<M>(s).unwrap()
}
//Parse json string into protobuf byte sequence
pub fn transform_json_to_protobuf<M: MessageFull>(s: &str) -> Vec<u8> {
    parse_message_from_json::<M>(s).write_to_bytes().unwrap()
}

//Parse protobuf byte sequence into json string
pub fn transform_protobuf_to_json<M: MessageFull>(p: &[u8]) -> String {
    let value = M::parse_from_bytes(p).unwrap();
    protobuf_json_mapping::print_to_string_with_options(
        &value,
        &PrintOptions {
            enum_values_int: true,
            proto_field_name: true,
            always_output_default_values: true,
            _future_options: (),
        },
    )
    .unwrap()
}
