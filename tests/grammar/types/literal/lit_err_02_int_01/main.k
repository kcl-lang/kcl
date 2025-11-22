schema CheckLiteralType:
    int_01: 123 | 456
    int_02: 123 | 456

x = CheckLiteralType {
    int_01: 123
    int_02: 789
}
