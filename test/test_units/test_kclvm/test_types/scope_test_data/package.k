import scope_test_data.pkg as pkg1
import scope_test_data.pkg.pkg as pkg2

person1 = pkg1.Person {
    name: "Alice"
    age: 18
}

person2 = pkg2.Person {
    name: "Alice"
    age: 18
}
