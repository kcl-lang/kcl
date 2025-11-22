schema TeamSpec[id: str]:
    fullName: str
    name: str = id
    shortName: str = name

schema Team:
    d: TeamSpec = TeamSpec("d")

Teams = Team {
    d = {
        fullName = "charlie"
        shortName = "dd"
    }
}
