"""The `protobuf` module defines functionality for parsing protocol buffer
definitions and instances.

Proto definition mapping follows the guidelines of mapping Proto to JSON as
discussed in https://developers.google.com/protocol-buffers/docs/proto3, and
carries some of the mapping further when possible with KCL.

The following type mappings of definitions apply:

    Proto type        KCL type/def     Comments
    message           schema           Message fields become KCL attribute, whereby
                                       names are mapped to lowerCamelCase.
    enum              e1 | e2 | ...    Where ex are strings. A separate mapping is
                                       generated to obtain the numeric values.
    oneof             e1 | e2 | ...    KCL union type.
    map<K, V>         {K:V}
    repeated V        [V]
    bool              bool
    string            str
    bytes             str              A base64-encoded string when converted to JSON.
    int32, fixed32    int              An integer with bounds as defined by int.
    uint32            int              An integer with bounds as defined by int.
    int64, fixed64    int              An integer with bounds as defined by int.
    uint64            int              An integer with bounds as defined by int.
    float             float            A number with bounds as defined by float.
    double            float            A number with bounds as defined by float.
    Struct            schema
    Value             any
    google.proto.Any  any
    ListValue         [...]
    NullValue         any
    BoolValue         bool
    StringValue       str
    NumberValue       float
    StringValue       str
    Empty             any
    Timestamp         float
    Duration          int

Protobuf definitions can be annotated with KCL constraints that are included
in the generated KCL:
    (kcl.opt)     FieldOptions
        val          str           KCL attribute default value.
        optional     bool          Whether the KCL attribute is optional
        final        bool          Whether the KCL attribute is final

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

from .parser import (
    Import,
    Package,
    Option,
    Field,
    OneOfField,
    OneOf,
    Map,
    Reserved,
    Range,
    EnumField,
    Enum,
    Message,
    Service,
    Rpc,
    Proto,
    ImportOption,
    Type,
    KeyType,
    parse_code,
)
from .protobuf import (
    protobuf_to_kcl,
)

__all__ = [
    "Import",
    "Package",
    "Option",
    "Field",
    "OneOfField",
    "OneOf",
    "Map",
    "Reserved",
    "Range",
    "EnumField",
    "Enum",
    "Message",
    "Service",
    "Rpc",
    "Proto",
    "ImportOption",
    "Type",
    "KeyType",
    "parse_code",
    "protobuf_to_kcl",
]
