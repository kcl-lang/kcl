schema Base:
    name: str

schema A(Base):
    bar: str

a = A {
    name: "base.A"
    bar: "xxx"
}
