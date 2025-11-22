i = 10
f = 1.1
s = i + f
t = f + i

schema Simple:
    f0: float
    f1: float
    f2: float

myschema = Simple {
    f0: f
    f1: s
    f2: t
}
