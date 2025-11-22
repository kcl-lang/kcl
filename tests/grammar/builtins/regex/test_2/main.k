import regex

_pattern = "^(?!-)[a-z0-9-]{1,63}(?<!-)$"
# True cases
x0 = regex.match("x", _pattern)
x1 = regex.match("x1", _pattern)
x2 = regex.match("6x", _pattern)
x3 = regex.match("x-7", _pattern)
# False cases
x4 = regex.match("$", _pattern)
x5 = regex.match("x" * 64, _pattern)
x6 = regex.match("_x2", _pattern)
x7 = regex.match("__X3__", _pattern)
x8 = regex.match("x_4", _pattern)
