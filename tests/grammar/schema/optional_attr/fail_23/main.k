schema Address:
    street: str
    city: str

schema Person:
    name: str
    address: Address

person = Person {
    name: "Alice"
    address: Address {
        city: "NYC"
    }
}