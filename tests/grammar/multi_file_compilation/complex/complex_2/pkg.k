schema PersonInfo[personVal]:
    info: str = personVal.name + " " + str(personVal.age)

info = PersonInfo(person) {}
