import regex

# True cases
x0 = regex.match("x", r"^[A-Za-z_][A-Za-z0-9_]*$")
x1 = regex.match("x1", r"^[A-Za-z_][A-Za-z0-9_]*$")
x2 = regex.match("_x2", r"^[A-Za-z_][A-Za-z0-9_]*$")
x3 = regex.match("__X3__", r"^[A-Za-z_][A-Za-z0-9_]*$")
x4 = regex.match("x_4", r"^[A-Za-z_][A-Za-z0-9_]*$")
# False cases
x5 = regex.match("$", r"^[A-Za-z_][A-Za-z0-9_]*$")
x6 = regex.match("6x", r"^[A-Za-z_][A-Za-z0-9_]*$")
x7 = regex.match("x-7", r"^[A-Za-z_][A-Za-z0-9_]*$")
