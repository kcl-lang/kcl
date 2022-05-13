schema Data:
    __settings__: {str:} = {
        "output_type": "STANDALONE"
    }
    id?: int
    name: str

x0 = [[{a: 1, b: Undefined}]]
x1 = [[[{a: 1, b: Undefined}]]]
x2 = [[{a: 1, b: None}]]
x3 = [[[{a: 1, b: None}]]]
x4 = [[[1, Data { name = "data" }]]]
