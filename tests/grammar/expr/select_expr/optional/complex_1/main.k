schema Person:
    name: str
    env: [{str:}] = [{
        "key1": "value1"
    }, {
        "key2": "value2"
    }]
    data: {str:} = {"name": {"key": "value"}}
    value: str = env?[0].key1 + env?[1].key2

person = Person {name: "Alice"}
name = person?.env?[0].key1 + person?.env?[1].key2
