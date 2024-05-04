schema Foo:
    foo: int = 1

schema Bar:
    bar: int = 1

foo: Foo | Bar = Foo{}
bar = foo as Foo
