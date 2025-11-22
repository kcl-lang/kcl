import pkg

schema Son(pkg.B):
    field: str
    check:
        field == "123"

son = Son {
    fields: "asa",
    field: '123'
}
