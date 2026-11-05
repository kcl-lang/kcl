import units

type Ints = [int]

schema Item:
    value: int
    optional?: [int]

_float = lambda xs: [float] { xs }
_alias = lambda xs: Ints { xs }
_union = lambda xs: [int | str] { xs }
_literal = lambda xs: [1] { xs }
_units = lambda xs: [units.NumberMultiplier] { xs }
_keys = lambda d: {int:int} { d }
_schema = lambda xs: [any], d: {str:any} {
    assert typeof(xs[0]) == "Item"
    assert typeof(d.item) == "Item"
    xs[0].value = 2
    xs
}
_shallow = lambda xs: [any] {
    xs[0][0] = 2
    xs
}
_deep = lambda xs: [[int]] {
    xs[0][0] = 2
    xs
}

_items = [Item {value = 1}]
_nested = [[1]]
_keyed: any = {"1": 2}
_deep_copy = _deep(_nested)
assert _deep_copy == [[2]]
assert _nested == [[1]]
_shallow_copy = _shallow(_nested)
assert _nested == [[2]]
_schema_copy = _schema(_items, {item = Item {value = 3}})
assert _items[0].value == 2
assert _float(None) is None
assert _float(Undefined) is Undefined
assert _float([None, Undefined]) == [None, Undefined]
assert typeof(_float([1])[0]) == "int"
assert _alias([1, 2]) == [1, 2]
assert _union([1, "s"]) == [1, "s"]
assert _literal([1]) == [1]
assert str(_units([1Ki])[0]) == "1Ki"
assert _keys(_keyed) == {"1": 2}
assert _alias(_items[0].optional) is Undefined

result = True
