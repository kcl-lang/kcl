b = 1
# Use (i) to prevent the schema expr parse conflict
data = [lambda x, y=(i), z=(b) {
    x + y + i
}(i) for i in [1, 2, 3]]
