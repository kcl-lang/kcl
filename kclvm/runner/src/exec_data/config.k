data = {
    key1: [0]
    key1 += [1]
    key2: [0]
    key2 = [1]
    key3 = [0]
    key3 = [1]
}

schema Config:
    data: [int]
    env: [{str:}] = [{key1: 1}, {key2: 2}]

config = Config {
    data = [1]
    data += [2]
    env[0]: {key2: 2}
}
