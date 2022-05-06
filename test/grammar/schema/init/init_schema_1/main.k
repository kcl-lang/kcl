schema Inner:
    data?: {str:str}

schema Person:
    inner?: {str:str} = None
    innerModel?: Inner = Inner {
        "data": {
            "key": "value"
        }
    }
    name?: str

JohnDoe = Person {
    "name": "Joe"
}
