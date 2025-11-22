schema Name:
    firstName: str
    lastName: str

schema Person:
    age: int
    name: Name
    role: str
    info: {str:}
    family: [str]
    adult: bool

schema Community:
    members: [Person]

John = Community {
    "members": [
        {
            "age": 15,
            "role": "monitor",
            "name": {"firstName": "John", "lastName": "Reese"},
            "info": {"address": "Beijing"},
            "family": ["mom", "dad"],
            "adult": False
        }
    ]
}
