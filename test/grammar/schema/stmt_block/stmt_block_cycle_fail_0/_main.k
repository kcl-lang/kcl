schema Son:
    son_field: str
    parent: Parent = None

schema Parent[name](Son):
    field: str = name
    son: Son = Son {
        parent: Parent(name) {
            "field": "123"
        }
    }

parent = Parent(name = "123") {}
