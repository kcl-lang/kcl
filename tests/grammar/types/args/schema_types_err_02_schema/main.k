import sub as pkg

schema CheckArgType[x: pkg.Person]:
    user: any = x

schema Person:
    name: str = "KCL"

_x = Person{
    name: "person"
}

x1 = CheckArgType(_x) {}
