data = [ord(c) for c in "\0\A"]

assert 'aa' == ("a" + "a")
assert '\0\0' == ("\0" + "\0")
assert '\0A' == '\0' + 'A'
