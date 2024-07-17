schema Person:
    info: {str:} = {"key1": "value1"}
    hc: [int] = [1, 2, 3]

person = Person {
    info = {"key2": "value2"}
}
personDeleteHc0 = {
    **person
    hc[0] = Undefined
}
