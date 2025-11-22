import pkg.info

schema Person:
    firstName: str
    cards: [{str:}]
    info0: info.info

    _spec = {
        "info_0": {
            "cards": cards
        }
    }
    info0 = info.info {
        "spec": _spec
    }

JohnDoe = Person {
    "firstName": "John",
    "cards": [info.card{"num": 123}]
}
