schema Config:
    label: str = "A"
    name: str
    if label == "A":
        name = "{}-{}".format(label, "A")
    elif label == "B":
        name = "{}-{}".format(label, "B")
    else:
        name = "{}".format(label)
    anotherName: str = name

configA = Config {}
configB = Config {
    label = "B"
}
configC = Config {
    label = "C"
}

