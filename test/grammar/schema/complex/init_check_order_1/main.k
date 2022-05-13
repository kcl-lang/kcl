
schema Name:
    mixin [UpperMixin]
    firstName: str
    lastName: str
    upper: str

    # print("init name")

schema Person(Name):
    gender: str
    title: str
    info: str

    # print("init person")

schema Girl(Person):
    mixin [TitleMixin, InfoMixin]
    gender: str = "female"
    added: str = "some girl attr"

    # print("init girl")

    check:
        gender == "female", "gender should be female in Girl"

schema Boy(Person):
    mixin [TitleMixin, InfoMixin]
    gender: str = "male"
    added: str = "some boy attr"

    # print("init boy")

    check:
        gender == "male", "gender should be male in Boy"

schema UpperMixin:
    # print("init upperMixin")
    upper: str = lastName.upper()

schema TitleMixin:
    # print("init title mixin")
    if gender == "female":
        title = "Ms.{}".format(lastName)
    else:
        title = "Mr.{}".format(lastName)

schema InfoMixin:
    # print("init info mixin")
    info = "{}, {}".format(title, gender)

alice = Girl {
    "firstName": "Alice",
    "lastName": "Smith"
}

# print(" ===")

bob = Boy {
    "firstName": "Bob",
    "lastName": "Green"
}
