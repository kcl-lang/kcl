import .app

schema Person:
    name: str

list = [
    Person {name: "hello1"}
    Person {name: "hello2"}
]

result = [app.name for app in list]
