import ..pkg

schema Person3:
    p1: pkg.Person
    p2: [pkg.Person]
    p3: {str:pkg.Person}
    p4: pkg.Person | pkg.Person1

