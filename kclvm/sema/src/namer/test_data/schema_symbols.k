import import_test.a
import import_test.b
import import_test.c
import import_test.d
import import_test.e
import import_test.f as g
import pkg
import regex

schema Main(d.Parent):
    mixin [c.TestOfMixin]
    name?: str
    age?: int = 18
    person?: a.Person
    list_union_type: [e.UnionType|int]
    dict_union_type: {g.UnionType|int:float}

    check:
        regex.match(name, r"[a-z0-9]([-a-z0-9]*[a-z0-9])?(\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*") if name

if a._a > 1:
    _c = 1
elif a._a == 1:
    _c = 2
else:
    _c = 3

p = Main{
    age = b._b
}

person = pkg.Person {}
