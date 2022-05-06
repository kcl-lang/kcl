import pkg

schema Person(pkg.Person):
    hc?: int = None

personPkg = pkg.Person {}
person = Person {}

instancesTypeofStr = [typeof(inst) for inst in pkg.Person.instances()]
instancesFullTypeofStr = [typeof(inst, full_name=True) for inst in pkg.Person.instances()]
