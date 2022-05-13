# Copyright 2020 The KCL Authors. All rights reserved.

import google.protobuf.json_format as json_format


def FromJson(text: str, message):
    return json_format.Parse(text, message)


def ToJson(message) -> str:
    return json_format.MessageToJson(
        message, including_default_value_fields=True, preserving_proto_field_name=True
    )
