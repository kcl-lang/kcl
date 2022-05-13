schema Person:
    name: str

func: any = lambda x: Person {
    x.name
}
name = func(Person {name = 1})
