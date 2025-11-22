schema Foo:
    foo: str = "foo"

schema Bar:
    bar: str = "bar"

factory = lambda type: any, attrs: {:} = {} -> Foo | Bar {
    assert typeof(type) == "type"
    func = $type
    instance = func() {**attrs}
}

_foo = factory(Foo)
_bar = factory(Bar)
if typeof(_foo) == "Foo":
    foo = _foo as Foo

if typeof(_bar) == "Bar":
    bar = _bar as Bar
