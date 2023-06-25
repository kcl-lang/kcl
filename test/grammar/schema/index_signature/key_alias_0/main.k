schema TeamSpec[id: str]:
    fullName: str
    name = id
    shortName: str = name

schema TeamMap:
    [n: str]: TeamSpec = TeamSpec(n)  # `n` does not work currently

teamMap = TeamMap {
    a.fullName = "alpha"
    b.fullName = "bravo"
}
