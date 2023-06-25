schema TeamSpec:
    fullName: str
    name = id
    shortName: str = name

schema TeamMap:
    [n: str]: TeamSpec = TeamSpec {
        name = n
    }

teamMap = TeamMap {
    a.fullName = "alpha"
    b.fullName = "bravo"
}
