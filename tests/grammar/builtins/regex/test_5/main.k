import regex

keys = regex.findall("a=1 b=2", r"(\w)=\d")
optional = regex.findall("b", r"(a)?b")
either = regex.findall("ab", r"(a)|(b)")
