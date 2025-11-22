schema Parent(Son):
    parent_field: str

schema Son(GrandSon):
    son_field: str

schema GrandSon(Parent):
    grandson_field: str

parent = Parent {
    parent_field: ""
}
