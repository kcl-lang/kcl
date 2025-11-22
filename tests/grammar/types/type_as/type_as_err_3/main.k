import base
import child

schema A(base.Base):
    name: str
    main: str

a = A {
    name: "main.A"
    main: "123"
}

type AA = child.A

# Cast to / used for skip compile-time type check
_child_a: base.Base = child.a
# Must fail at runtime: typeof(_child_a) == 'child.A'
child_a = _child_a as A
child_a_type = typeof(_child_a,True)
