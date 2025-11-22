@deprecated
schema ObsoleteSchema:
    attr?: str

schema Person:
    name: str = "John"
    attrs: ObsoleteSchema = ObsoleteSchema {}

person = Person {}
