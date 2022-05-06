schema Person:
    [str]: str
    firstName : str = "John"
    lastName : str

JohnDoe = Person {
    "lastName": "Doe",
    "fullName": "Tom Cat"
}
