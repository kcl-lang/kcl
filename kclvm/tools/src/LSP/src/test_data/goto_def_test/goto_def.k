import .pkg
import .pkg.schema_def
    
p = pkg.Person {
    name: "alice"
    age: 1
}

p1 = p

schema Person3:
    p1: pkg.Person
    p2: [pkg.Person]
    p3: {str: pkg.Person}
    p4: pkg.Person | pkg.Person1


schema Name:
    name: str

schema Person:
	n: Name

p2 = Person {
    n: Name{
        name: pkg.m.name
    }
}

s = p2.n.name

a: int = 1

schema Reviewer:
    teams?: [int]
    users?: [int]

reviewers: [Reviewer] = [{team: [1]}]

schema Fib:
    n1 = n - 1
    n2 = n1 - 1
    n: int
    value: int

    if n <= 1:
        value = 1
    elif n == 2:
        value = 1
    else:
        value = Fib {n = n1}.value + Fib {n = n2}.value
fib8 = Fib {n = 8}.value

schema_map: {str: Person} = {
    person: p2
    person1: p2
}

p3 = schema_map.person.n.name