type Int = int
type Float = float
type Bool = bool
type String = str
type Any = any

schema Data:
    a?: Int = 1
    b?: Float = 2.0
    c?: Bool = True
    d?: String = "xx"
    e?: Any = None

data = Data {}
