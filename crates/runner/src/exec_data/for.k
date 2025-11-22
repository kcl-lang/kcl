x0 = [i for i in [1, 2, 3] if i > 1]
x1 = [i if i > 1 else i + 1 for i in [1, 2, 3]]
x2 = [i + j for i, j in [1, 2, 3]]

x3 = {k: k for k in {key1: "value1", key2: "value2"}}
x4 = {k: k for k in {key1: "value1", key2: "value2"} if k == "key1"}
x5 = {k: v for k, v in {key1: "value1", key2: "value2"}}
