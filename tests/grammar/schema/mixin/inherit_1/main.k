schema FullNameMixin:
    fullName = "{} {}".format(firstName, lastName)

schema AcademyFullNameMixin:
    academyFullName = "{} {}".format(school, subject)

schema PersonInfoMixin:
    info = "{}, {} years old".format(fullName, age)

schema Name:
    mixin [FullNameMixin]
    firstName: str
    lastName: str
    fullName: str

schema Person(Name):
    mixin [PersonInfoMixin]
    age: int
    info: str

schema Scholar(Person):
    mixin [AcademyFullNameMixin]
    subject: str
    school: str
    academyFullName: str

JohnReese = Name {
    "firstName": "John",
    "lastName": "Reese"
}

JonSnow = Person {
    "firstName": "Jon",
    "lastName": "Snow",
    "age": 10
}

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "age": 10,
    "subject": "CS",
    "school": "PKU"
}
