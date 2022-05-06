_family = [{
    "name": "John",
    "relation": "father"
}]
afterFormat = [{
    person["relation"]: person["name"]
} for person in _family]
