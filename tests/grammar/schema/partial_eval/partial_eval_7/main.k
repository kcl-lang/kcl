schema TeamSpec:
    fullName?: str
    name?: str = Undefined
    shortName?: str = name

schema TeamMap:
    a: TeamSpec = TeamSpec {
        name = "a"
    }

teamMap = TeamMap {
    a.fullName = "alpha"
}
