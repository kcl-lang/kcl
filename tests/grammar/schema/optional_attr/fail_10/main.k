schema TeamSpec[id: str]:
    fullName: str
    name: str = id
    shortName: str = name

schema Team:
    [id: str]: TeamSpec = TeamSpec(id)

Teams = Team {
    a.fullName = "alpha"
    b.fullName = "bravo"
    c = {
        fullName = "charlie"
        shortName = "cc"
    }
}
