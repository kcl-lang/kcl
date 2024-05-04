schema Foo:
    foo: str

schema Bar:
    bar: str

factory = lambda type: any, attrs: {:} = {} -> Foo | Bar {
    assert typeof(type) == "type"
    func = $type
    instance = func() {**attrs}
}

_foo = factory(Foo, {foo = "foo"}) # Note we set attributes here.
_bar = factory(Bar, {bar = "bar"}) # Note we set attributes here.
if typeof(_foo) == "Foo":
    foo = _foo as Foo

if typeof(_bar) == "Bar":
    bar = _bar as Bar
