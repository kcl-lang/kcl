import temp

schema FinalSchema:
    _values: [temp.MySchema] = [{id: "hello"},{ id: "world" },{}]
    finalValues: [temp.MySchema] = [v {someField = i} for i, v in _values]

f = FinalSchema{}
