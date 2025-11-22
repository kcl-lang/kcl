schema Base:
    info: str = "BaseInfo"

schema Person(Base):
    name: str = "Alice"
    age: int

person = Person {
    age: 18
}
