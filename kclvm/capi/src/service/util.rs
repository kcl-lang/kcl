use protobuf::MessageFull;
use protobuf_json_mapping::PrintOptions;

/// Parse byte sequence into protobuf message
pub fn parse_message_from_protobuf<M: MessageFull>(p: &[u8]) -> Result<M, protobuf::Error> {
    M::parse_from_bytes(p)
}

/// Parse json string into protobuf message
pub fn parse_message_from_json<M: MessageFull>(
    s: &str,
) -> Result<M, protobuf_json_mapping::ParseError> {
    protobuf_json_mapping::parse_from_str::<M>(s)
}

/// Parse protobuf byte sequence into json string
pub fn transform_protobuf_to_json<M: MessageFull>(
    p: &[u8],
) -> Result<std::string::String, protobuf_json_mapping::PrintError> {
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
}
