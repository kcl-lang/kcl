schema Foo[a : int]:
    bar? : int

f = lambda x {}

foo = Foo(1,2,3)
f(1,2)

schema Foo2[a : int, b: int]:
    bar? : int

foo2 = Foo2(1,2,3)