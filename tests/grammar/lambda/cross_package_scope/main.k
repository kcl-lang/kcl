import helpers

# Lazy resolution and callbacks cross the package stack in both directions.
first = helpers.apply(_callback, 2)
_callback = lambda x: int -> int {
    local = x + _offset
    local
}
_offset = 10

schema Item:
    value: int = helpers.apply(_callback, first)

output = Item {}
_values = [first]
_values += [output.value]
assert _values == [15, 28]
