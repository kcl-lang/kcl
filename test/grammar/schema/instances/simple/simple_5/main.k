import pkg

schema Person(pkg.Person):
    hc?: int = None

personPkg = pkg.Person {}
person = Person {}

personPkgInstanceCount = len(pkg.Person.instances())
personInstanceCount = len(Person.instances())
