data = {
    k1: "123"
    k2: "456"
}
x0 = [1 if all c in v {c == "1"} else 0 for k, v in data]
y0 = [1 if any c in v {c == "1"} else 0 for k, v in data]
x1 = [1 for k, v in data if all c in v {c == "1"}] or [0]
y1 = [1 for k, v in data if any c in v {c == "1"}]
