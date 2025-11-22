
schema Person:
    name: str
    title: str
    gender: str

    check:
        name.istitle(), "name of person must be titled"

schema Girl(Person):
    mixin [TitleMixin]
    gender: str = "female"

    check:
        gender.endswith("male"), "gender must be in *male format"

schema TitleMixin:
    if gender == "female":
        title = "Ms.{}".format(name)
    else:
        title = "Mr.{}".format(name)

    check:
        gender.endswith("male"), "gender must be in *male format"

alice = Girl {
    "name": "Alice"
}
