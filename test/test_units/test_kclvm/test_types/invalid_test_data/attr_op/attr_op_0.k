a = {
    key1 = [0]  # [0]
    key1 += [1]  # [0, 1]
    key1 += None  # Error: only list type can in inserted
}
