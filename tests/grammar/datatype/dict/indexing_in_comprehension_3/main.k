schema Family:
    father?: str = None
    mother?: str = None

schema Person:
    family?: [Family]

schema Front:
    family?: [{str:}]

schema Wrapper[data]:
    person?: Person = {
        family = [Family {
            "${p.relation}" = p.name
        } for p in data.family]
    }

result = Wrapper(Front {
    family: [{
        name = "John",
        relation = "father"
    }]
}) {}
