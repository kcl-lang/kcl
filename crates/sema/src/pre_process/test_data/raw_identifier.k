$schema = 1

schema Name:
    $name?: str
    "$name"?: str

n = Name {
    "$name": "1"
}
