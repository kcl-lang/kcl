print("setup")

schema Item:
    n: int

items = [Item {n = i} for i in range(3)]

double = lambda x: int -> int {
    x * 2
}
