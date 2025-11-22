schema Person:
    key: [int]

base = {"key": [1, 2, 3]}

person = Person {
    **base
    key = [4, 5, 6]
}
