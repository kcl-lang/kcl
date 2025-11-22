schema Base:
    name: str

schema Person(Base):
    name?: str
    age?: int

person = Person {
    name: "Alice"
    age: 18
}
