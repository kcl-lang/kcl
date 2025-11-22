schema Parent(Son):
    parent_field: str
    son: Son = Son {
        parent: Parent {
            parent_field: "123"
        }
    }

schema Son:
    son_field: str
    parent: Parent = None

parent = Parent {
    parent_field: "",
}
