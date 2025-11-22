schema Derived1(Base):
    name: str = "1"

schema Base:
    name: str
    name2 = name

schema Derived2(Base):
    name: str = "2"

d1 = Derived1 {}
d2 = Derived2 {}
