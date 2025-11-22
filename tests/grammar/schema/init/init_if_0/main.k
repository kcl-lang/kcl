schema Person:
    config: {str:str}
    name: str

    if config and config["name"]:
        name = config["name"]

JohnDoe = Person {
    "config": {
        "name": "John Doe"
    }
}
