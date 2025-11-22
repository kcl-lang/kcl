schema Person:
    firstName: str = "John"
    midName: str = "Jon"
    lastName: str
    image: str
    image_pull_policy: str
    check:
        image not None and image_pull_policy not None, "oh no, {}".format("we fail here")
        image not None and image_pull_policy not None
        image_pull_policy in ["Always", "Never", "IfNotPresent"]
        len(lastName)
        (lastName not None)
        lastName not None
        image_pull_policy not None if image
        image_pull_policy if image
        len(lastName) > 0
        len(lastName), "oh no, {}".format("we fail here")

JohnDoe = Person {
    "lastName": "Doe",
    "image": "docker.image",
    "image_pull_policy": ""
}
