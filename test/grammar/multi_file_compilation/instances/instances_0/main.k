schema Base:
    name: str
    age: int

schema Person(Base):
    name: str = "Alice"
    age: int = 18

person = Person {}
