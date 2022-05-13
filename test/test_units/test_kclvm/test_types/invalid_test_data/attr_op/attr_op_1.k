schema Data:
    key1: [int]

a = Data {
    key1 = [0]  # [0]
    key1 += [1]  # [0, 1]
    key1 += Undefined  # Error: only list type can in inserted
}
