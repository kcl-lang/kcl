schema Person:
    name?: str

    check:
        name, "name should be defined"
