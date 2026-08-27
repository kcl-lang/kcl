schema Item:
    p: int
    q: int

schema Nested:
    a: int
    b: int

schema Mid:
    a: int
    nested: any
    c: int

# A config entry must shadow an attribute of the same name in the enclosing
# schema for the entries that follow it. See issue #1769.
schema Outer:
    p: int
    item: any = Item {
        p: p + 1
        q: p + 1
    }
    # The shadowing must not outlive the config expression.
    after: int = p

# The shadowing must survive the evaluation of a nested schema body and must be
# restored to the enclosing config entry when that nested config is left.
schema Top:
    a: int
    mid: any = Mid {
        a: a + 1
        nested: Nested { a: 50, b: a }
        c: a
    }

outer = Outer { p: 100 }
top = Top { a: 10 }
