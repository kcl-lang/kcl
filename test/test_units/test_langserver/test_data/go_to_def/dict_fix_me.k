schema Person:
    name: Name
    age: int
    additional: {str: Info}

schema Info:
    value: str

schema Name:
    first: str
    last: str

alice: Person = {
    name: {
        first: "alice"
        last: "Green"
    }
    age: 10
    additional: {
        hobby: {
            value: "piano"
        }
    }
}