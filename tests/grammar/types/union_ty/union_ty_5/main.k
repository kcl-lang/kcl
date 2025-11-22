schema A:
    a: int

schema X:
    x: [A | "1"]
    y: ["2" | A]

x = X {
    x: [{a = 1}]
    y: [{a = 2}]
}
