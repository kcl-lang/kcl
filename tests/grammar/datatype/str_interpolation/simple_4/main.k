a = 1
b = 1.2
c = True
d = [1]
e = {"key": "value"}
data = "${a + 1 if a else b} ${b - 1 + int(c)} ${int(c) + 3} ${(d * 2)[0]} ${e['key']}"
if a == 1:
    raw = data + " ${a}" + str(a)
