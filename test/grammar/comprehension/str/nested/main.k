dataString = "abc"
dataList = [
    "{} {}".format(c1, c2) \
    for c1 in dataString.lower() \
    for c2 in dataString.upper()
]
