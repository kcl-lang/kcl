import json
import yaml

# Issue #1763: JSON-decoded `null` (and YAML-decoded `null`) must override
# a schema attribute's default value. Previously, json/yaml decode inserted
# keys with `Union`, which caused `value_union`'s `is_none_or_undefined`
# short-circuit to drop the explicit `null` in favour of the default.
schema Temp:
    x: int
    y?: int = x

# KCL literal `None` works (override syntax).
correct: Temp = {x = 1} | {y = None}
# JSON-decoded `null` must produce the same result.
from_json: Temp = {x = 1} | json.decode('{"y": null}')
# YAML-decoded `null` (which delegates to the JSON decoder) as well.
from_yaml: Temp = {x = 1} | yaml.decode("y: null\n")
# Inside a nested object too.
nested: Temp = {x = 1} | json.decode('{"y": null}')
