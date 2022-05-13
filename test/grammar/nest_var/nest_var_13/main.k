base = {data.key: [1, 2, 3]}

data = {
    **base
    if True:
        data.key = [4, 5, 6]
}
