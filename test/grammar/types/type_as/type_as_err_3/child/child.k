import base

schema A(base.Base):
    foo: str

a = A {
    name: "child.A"
    foo: "test"
}

# Cast to  used for skip compile-time type check
_base_a: base.Base = base.a

# Must fail at runtime: typeof(_base_a) == 'base.A'
base_a: A = _base_a as A
base_a_type = typeof(base_a, True)
child_a_type = typeof(a, True)
