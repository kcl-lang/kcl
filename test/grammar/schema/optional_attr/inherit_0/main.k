schema Base:
    info: str

schema Person(Base):
    name?: str
    age?: int

person = Person {
    info: "PersonInfo"
    name: "Alice"
    age: 18
}
