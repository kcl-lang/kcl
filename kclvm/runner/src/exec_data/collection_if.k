schema Config:
    name: str
    env: str

env = "env"

data1 = Config {
    if env == "env":
        name: env
        env: env
    else:
        name: "name"
        env: "name"
}

data2 = Config {
    if env != "env":
        name: env
        env: env
    else:
        name: "name"
        env: "name"
}

data3 = {
    if True:
        key1: "value1"
    elif True:
        key2: "value2"
    elif True:
        key3: "value3"
    else:
        key4: "value4"
}

data4 = [
    if True:
        "value1"
    elif True:
        "value2"
    elif True:
        "value3"
    else:
        "value4"
]
data5 = [
    if True: "value1"
]
data6 = {
    if True: key: "value1"
}
data7 = [
    if False: *[0]
    elif True: *[1]
    else: *[2]
]
data8 = {
    if False: **{k1: "v1"}
    elif False: **{k2: "v2"}
    else: **{k3: "v3"}
}
