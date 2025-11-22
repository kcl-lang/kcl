data = {
    k1: ["1", "2", "3"]
    k2: ["4", "5", "6"]
}
x0 = [1 if filter c in v {c == "1"} else 0 for k, v in data]
y0 = [1 if map c in v {c == "1"} else 0 for k, v in data]
x1 = [1 for k, v in data if filter c in v {c == "1"}]
y1 = [1 for k, v in data if map c in v {c == "1"}]
