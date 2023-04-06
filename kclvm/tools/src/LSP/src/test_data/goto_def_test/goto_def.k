import .pkg
import .pkg.schema_def
    
p = pkg.Person {
    name: "alice"
    age: 1
}

p1 = p

schema Person3:
    p1: pkg.Person
    p2: [pkg.Person]
    p3: {str: pkg.Person}
    p4: pkg.Person | pkg.Person1