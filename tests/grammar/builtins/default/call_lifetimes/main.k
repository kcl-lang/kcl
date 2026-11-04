import json
import runtime

_data = {items = ["kept", "alive"]}
_alias = option("call_lifetimes_data", default=_data)
_method = option("call_lifetimes_method", default="xy".replace)
_methods = list([_method])
_caught = runtime.catch(lambda {
    _method()
})

schema Entry:
    name: str

_entry = Entry {name = "kept"}

shared = _alias
method = _methods[0]("x", "z")
roundtrip = json.decode(json.encode(_alias))
instances = [entry.name for entry in Entry.instances()]
caught = bool(_caught)
