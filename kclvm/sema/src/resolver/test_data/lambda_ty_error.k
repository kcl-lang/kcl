_func = lambda x: int, y: int -> int {
    x - y
}  # Ok
_func = lambda x: int, y: int -> str {
    str(x + y)
}  # Error: expect function (int, int) -> str, got function (int, int) -> int