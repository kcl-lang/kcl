schema CheckLiteralType:
    int_01: 123K | 456
    int_02: 123 | 456K

x = CheckLiteralType {
    int_01 = 123
    int_02 = 123
}
