schema Foo:
    foo: int

schema Bar:
    bar: int

foo: Foo = Bar {
    bar: 1
}
