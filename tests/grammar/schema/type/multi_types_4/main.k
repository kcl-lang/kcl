schema Person:
    name: {str:str} | str

alice = Person {
    name: "alice"
}

aliceGreen = Person {
    name: {
        first: "alice",
        second: "Green"
    }
}
