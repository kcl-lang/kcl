schema Parent(Son):
    parent_field: str

schema Son(Parent):
    son_field: str

parent = Parent {
    parent_field: ""
}
