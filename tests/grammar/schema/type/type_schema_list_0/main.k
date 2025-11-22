schema Person:
    name: str = "Alice"

schema Table:
    persons: [Person] = [Person {}, Person {}]

table = Table {}
