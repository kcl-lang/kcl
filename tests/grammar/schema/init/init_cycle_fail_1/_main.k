schema Parent(Son):
    parent_field: str
    son: Son = Son {
        parent: Parent {
            parent_field: "123"
        }
    }

schema Son:
    son_field: str
    parent: Parent = Parent {
        son: Son {
            son_field: "123"
        }
    }

parent = Parent {
    parent_field: "",
}
