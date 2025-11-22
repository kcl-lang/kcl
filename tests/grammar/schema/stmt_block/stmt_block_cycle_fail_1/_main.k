schema Parent:
    field: str = "123"
    son: Son = Son {
        parent: Parent {
            "field": "123"
        }
    }

schema Son(Parent):
    son_field: str
    parent: Parent = Parent {}

parent = Parent {
    "field" = "name"
}
