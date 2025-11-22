schema Config:
    val: [{str:}]

base = {
    val: [{key1: "value1"}]
}
config = {
    **base
    val: [{key2: "value2"}]
}
