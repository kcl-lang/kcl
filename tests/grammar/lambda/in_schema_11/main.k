protocol StringProtocol:
    s: str

mixin StringMixin for StringProtocol:
    add = lambda pref: str {
        pref + s
    }

schema String:
    mixin [StringMixin]
    s: str
    add: (str) -> str

s1 = String { s: "hello" }
s2 = String { s: "world" }

output = {
    s1_add: s1.add("foo ")
    s2_add: s2.add("bar ")
}
