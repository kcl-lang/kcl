schema card:
    num: str

    check:
        num not None

schema info:
    spec: info_spec

schema info_spec:
    info_0: info_0

schema info_0:
    cards: [card]

schema Person:
    firstName : str
    cards: [{str:}]
    info0: info

    _spec: {str:} = {
        "info_0": {
            "cards": cards
        }
    }

    info0 = info {
        "spec": _spec
    }

JohnDoe = Person {
    "firstName": "John",
    "cards": [card {"num":123}]
}
