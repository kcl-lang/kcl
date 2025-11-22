import base

schema A(base.Base):
    foo: str

a = A {
    name: "child.A"
    foo: "test"
}
