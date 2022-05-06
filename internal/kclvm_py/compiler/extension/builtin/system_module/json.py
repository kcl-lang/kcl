from pathlib import Path
import json

from .util import filter_fields


def KMANGLED_encode(
    data, sort_keys=False, indent=None, ignore_private=False, ignore_none=False
):
    return json.dumps(
        filter_fields(data, ignore_private, ignore_none),
        sort_keys=sort_keys,
        indent=indent,
    )


def KMANGLED_decode(value: str):
    return json.loads(value)


def KMANGLED_dump_to_file(
    data,
    filename: str,
    sort_keys=False,
    indent=None,
    ignore_private=False,
    ignore_none=False,
):
    json_str = KMANGLED_encode(data, sort_keys, indent, ignore_private, ignore_none)
    Path(filename).write_text(json_str)
