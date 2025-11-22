schema Inner:
    data: {str:str}

schema Person:
    inner: {str:str}
    innerModel: Inner
    name: str

    if inner:
        innerModel = Inner {
            "data": inner
        }

JohnDoe = Person {
    "name": "Joe",
    "inner": {
        "key": "value0"
    }
}
