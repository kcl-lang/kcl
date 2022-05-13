schema Person:
    firstName: str = "John"
    midName: str = "Jon"
    lastName: str
    image: str
    image_pull_policy: str
    check:
        image is not None and image_pull_policy is not None, "oh no, {}".format("we fail here")
        image is not None and image_pull_policy is not None
        image_pull_policy in ["Always", "Never", "IfNotPresent"]
        len(lastName)
        (lastName is not None)
        lastName is not None
        (image_pull_policy is not None) if image
        image_pull_policy if image
        len(lastName) > 0
        len(lastName), "oh no, {}".format("we fail here")

JohnDoe = Person {
    "lastName": "Doe",
    "image": "docker.image",
    "image_pull_policy": "Never"
}
