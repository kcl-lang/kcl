# Regression test for issue #1759: a top-level field forced early by a forward
# reference must be evaluated exactly once. Each `show(...)` prints a unique
# key line; a second evaluation would emit a duplicate key and change the
# output. `_foo` is used before its declaration (forward reference) and `_bar`
# after it.
show = lambda s: str -> str {
    print("eval_${s}: seen")
    s
}

foo = _foo

_foo = {
    name = "foo"
    value = show("foo")
}

_bar = {
    name = "bar"
    value = show("bar")
}

bar = _bar
