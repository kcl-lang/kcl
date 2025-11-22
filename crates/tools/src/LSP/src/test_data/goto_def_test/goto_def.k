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

reviewers: [Reviewer] = [Reviewer {teams: [1]}]

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

params = option("params")
toMatch = params.toMatch
toAdd = params.toAdd
items = [item | {
   # If all annotations are matched, patch more annotations
    if all key, value in toMatch {
        item.metadata.annotations[key] == value
    }:
       metadata.annotations: toAdd
} for item in option("items")]


capabilities = option("params").capabilities or ["SETUID", "SETFCAP"]
items1 = [item | {
    if item.kind == "Pod":
        spec.containers: [{
            "securityContext": {"capabilities": {"add" += [cc] if cc not in (container?.securityContext?.capabilities?.drop or []) else [] for cc in capabilities}}
        } for container in item.spec.containers]
} for item in option("items")]

p4 = Person {
    n.name: "a"
}

func = lambda x: int, y: int -> int {
    x + y
}

b = True
command: [str] = [
    if b:
        "a"
]

f = lambda a: [str], b: [str], c: [str] -> [str] {
    c + a + b
}
