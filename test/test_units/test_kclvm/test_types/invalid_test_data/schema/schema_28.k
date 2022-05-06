# Arguments cannot be used in the schema modification expression
schema Person:
    name: str

personOrigin = Person { name = "Alice" }
personNew = personOrigin("invalid arguments") { name = "Bob" }
