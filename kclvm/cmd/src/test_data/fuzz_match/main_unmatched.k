schema Person:
    en?: int
    sd?: int

p = Person {}
a = p.a # Error, attribute 'a' not found in `Person`