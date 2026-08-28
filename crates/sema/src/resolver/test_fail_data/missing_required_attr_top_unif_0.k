# Case A: top-level unification across statements.
# The two statements are merged into a single `Person {age = 10; name = "Alice"}`
# by the pre-processor, so the static check should NOT report `name` as missing.
schema Person:
    name: str
    age?: int

p: Person {
    age = 10
}
p: Person {
    name = "Alice"
}