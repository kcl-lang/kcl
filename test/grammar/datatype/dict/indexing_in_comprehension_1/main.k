schema Family:
    father?: str = None
    mother?: str = None

schema Person:
    family?: [Family]

schema Front:
    family?: [{str:}]

schema Wrapper[data]:
    person?: Person = {
        "family": [{
            p["relation"]: p["name"]
        } for p in data.family]
    }

f = Front {
    family: [{
        "name": "John",
        "relation": "father"
    }]
}

result = Wrapper(f) {}
