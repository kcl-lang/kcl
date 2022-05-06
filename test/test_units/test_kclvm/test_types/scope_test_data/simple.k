schema Base:
	id0: str = 'id-base'
	id1: str

schema Person[_name:str, id='id-person'](Base):
	name: str = 'kcl'
	age: int = 1
	x: 1 | "aaa" | True = 1

person = Person(_name="arg-name") {
	name: 'Alice'
}

none0 = Undefined
none1 = None
bool1 = True
bool2 = False
bool3: bool = None
s0 = 'abc'
s1: str = 'abc'
i0 = 123.3
f0 = 1.5
x0 = 1 + 1.5
x1 = x0 + 1
personOther: Person = Person {}
if person.name == 'Alice':
    print("hello KCL")

rule SomeRule:
	x0 > 0
