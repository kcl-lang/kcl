import pkg

schema Son(pkg.Parent):
    age: int

alice = Son {
    name: "alice"
    age: 10
}