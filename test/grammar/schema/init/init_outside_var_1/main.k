patchlist = [1, 2]
data = 1

schema Person:
    name: str
    list: [int] = [4, 5, 6]
    val: int = 2
    val = data
    list |= patchlist
    name = "Alice"

person_alice = Person {}
