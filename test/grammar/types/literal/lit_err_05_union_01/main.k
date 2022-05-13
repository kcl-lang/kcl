schema CheckLiteralType:
    x_1?: True|123|3.14|"abc"|[]|{str:}

x = CheckLiteralType {
    x_1: 443
}
