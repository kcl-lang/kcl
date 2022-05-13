schema Config:
    val: [int] = [0]

base = {
    val = [2]
}
config = {
    **base
    val = [1]
}
