_items = []
functionTest = lambda -> str {
    "value"
}
_items += [{
    name = "A"
}]

_role = []

if False:
    _items += [
        {name = "B"}
    ]


_items += [
  {
    name = "C"
    key = functionTest()
  }
]

items = _items
