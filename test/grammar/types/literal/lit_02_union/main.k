

schema CheckLiteralType:
    x_1?: True|123|3.14|"abc"|[]|{str:}
    x_2?: True|123|3.14|"abc"|[]|{str:}
    x_3?: True|123|3.14|"abc"|[]|{str:}
    x_4?: True|123|3.14|"abc"|[]|{str:}
    x_5?: True|123|3.14|"abc"|[]|{str:}
    x_6?: True|123|3.14|"abc"|[]|{str:}

    s_00: ""

x = CheckLiteralType {
    x_1: True
    x_2: 123
    x_3: 3.14
    x_4: "abc"
    x_5: [1, 2, "111"]
    x_6: {"a":1, "b":2}

    s_00: ''
}
