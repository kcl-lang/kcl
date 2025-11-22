import .pkg.inner as pkg_inner

schema Person:
    inner: {str:str}
    innerModel: pkg_inner.Inner
    name: str

    if inner:
        innerModel = pkg_inner.Inner {
            "data": inner
        }

JohnDoe = Person {
    "name": "Joe",
    "inner": {
        "key": "value0"
    }
}
