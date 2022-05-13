from io import StringIO
from pathlib import Path
import ruamel.yaml as yaml

from .util import filter_fields

_yaml = yaml.YAML()
_yaml.representer.add_representer(
    str,
    lambda dumper, data: dumper.represent_scalar(
        u"tag:yaml.org,2002:str", data, style="|"
    )
    if "\n" in data
    else dumper.represent_str(data),
)
# Convert None to null
_yaml.representer.add_representer(
    type(None),
    lambda dumper, data: dumper.represent_scalar(u"tag:yaml.org,2002:null", u"null"),
)


def KMANGLED_encode(data, sort_keys=False, ignore_private=False, ignore_none=False):
    buffer = StringIO()
    data_filtered = filter_fields(data, ignore_private, ignore_none)
    if sort_keys:
        sorted_dict = yaml.comments.CommentedMap()
        for k in sorted(data_filtered):
            sorted_dict[k] = data_filtered[k]
        _yaml.dump(sorted_dict, buffer)
    else:
        _yaml.dump(data_filtered, buffer)
    return buffer.getvalue()


def KMANGLED_decode(value: str):
    buffer = StringIO(value)
    data = yaml.safe_load(buffer)
    return data


def KMANGLED_dump_to_file(
    data, filename: str, sort_keys=False, ignore_private=False, ignore_none=False
):
    yaml_str = KMANGLED_encode(data, sort_keys, ignore_private, ignore_none)
    Path(filename).write_text(yaml_str)
