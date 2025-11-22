schema Person:
    name: str = "kcl"
    age: int = 1

person = Person {
    **{k: v for k, v in {
        name: None
        age: Undefined
    }}
}
