schema Person:
    info: {str:} = {"key1": "value1"}
    hc: [int] = [1, 2, 3]

person = Person {
    info = {"key2": "value2"}
    hc = [4, 5]
}
