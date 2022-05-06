_name_private = "Alice"
name_public = "Alice"

schema Person:
    name: str = name_public
    name = _name_private.lower()

alice = Person {}
