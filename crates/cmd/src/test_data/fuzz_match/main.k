schema Person:
    aa?: int
    aaa?: int

p = Person {}
a = p.a # Error, attribute 'a' not found in `Person`, did you mean `aa`?