schema TeamSpec:
    fullName: str
    name: str
    shortName: str = name

schema TeamMap:
    [n: str]: TeamSpec = TeamSpec {
        name = n
    }

teamMap = TeamMap {
    a.fullName = "alpha"
    b.fullName = "bravo"
}
