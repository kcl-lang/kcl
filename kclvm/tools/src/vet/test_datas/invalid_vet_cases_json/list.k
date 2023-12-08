schema User:
    name: str
    age: int
    message?: str

assert typeof(value) == "list"
assert all v in value {
    typeof(v) == "User"
}
