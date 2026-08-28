# Case D: nested unification inside a parent schema. After pre-processing,
# both `lead: Person {age = 10}` and `lead: Person {name = "Alice"}` collapse
# into a single merged config inside the `lead` entry, so the static check
# should NOT report `name` as missing.
schema Person:
    name: str
    age?: int

schema Team:
    lead: Person

t: Team {
    lead: Person {age = 10}
}
t: Team {
    lead: Person {name = "Alice"}
}