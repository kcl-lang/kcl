name = "Alice"

schema Data:
    charsStr: [str]

data = Data {
    charsStr = [c * 2 for c in name]
}
