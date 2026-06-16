mixin BrokenMixin:
    something: str = "abc"
    broken_1 = UndefinedSchema {}
    broken_2 = UndefinedSchema("foobar")
    broken_3 = UndefinedSchema

schema Foo:
    mixin [
        BrokenMixin
    ]

foo = Foo {}
