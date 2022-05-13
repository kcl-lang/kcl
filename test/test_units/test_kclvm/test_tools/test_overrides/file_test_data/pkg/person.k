import pkg.internal_pkg

schema Person:
    name: str = "kcl"
    age: int = 1
    data?: internal_pkg.Data
