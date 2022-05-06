import .pkg.v1

schema Person:
    inner: {str:str}
    innerModel: v1.Inner
    name: str

    if inner:
        innerModel = v1.Inner {
            "data": inner
        }

JohnDoe = Person {
    "name": "Joe",
    "inner": {
        "key": "value0"
    }
}
