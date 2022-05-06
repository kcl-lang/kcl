a = b = 1

schema Person:
    mixin [
        aMixin
    ]
    age : int = 1
    name : str = "1"

person = Person {
    age = 1
}