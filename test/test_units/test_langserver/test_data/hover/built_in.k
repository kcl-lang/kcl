import base64
import crypto

# str built in
a = "hello World".capitalize()
b = "hello world".islower()
c = "Hello World".islower()
d = "Hello world, Hello world".count("o")
e = "Hello world, Hello world".count("Hello", 5, 20)
f = "Hello World".endswith("rld")
g = "Hello World".endswith("rld", 0, 9)


# base 64
decode = base64.decode("MC4zLjA=")


# crypto
md5 = crypto.md5("ABCDEF")
sha = crypto.sha1("ABCDEF")


# regex
regex_result = regex.match("192.168.0.1", "^(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|[1-9])\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)$")


# default
h = max(80, 100, 1000)
i = max(-80, -20, -10)


# type of
none1 = typeof(None)

# option
j = option("a", default=42, help="set a value")

# deprecated
@deprecated(version="1.2.3", reason="schema-deprecated", strict=False)
schema Person:
    @deprecated(version="4.5.6", reason="attr-deprecated", strict=True)
    name: str