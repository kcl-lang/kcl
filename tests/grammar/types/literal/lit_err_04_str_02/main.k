schema CheckLiteralType:
    x_01: "TCP" | 'UDP'

x = CheckLiteralType {
    x_01: 443
}
