import sub as pkg

schema CheckArgType[x: pkg.Person]:
    user: any = x

schema Person:
    name: str = "KCL"

_x = pkg.Person{
    name = "person"
}

x1 = CheckArgType(_x) {}
