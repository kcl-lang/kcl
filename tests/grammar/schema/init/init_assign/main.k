schema Person:
    age: int = 10
    name: str = "John"
    info: {str:} = {"address": "abc", "old": False}
    family: [str] = ["mom", "dad"]
    adult: bool = False

John = Person {}
