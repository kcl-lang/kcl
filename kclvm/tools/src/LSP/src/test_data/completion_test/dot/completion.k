import .pkg.  # complete import path
import .pkg.subpkg
schema Person:
    name: str
    age: int

p = Person {
    name: "alice"
    age: 1
}

p1 = p.       # complete schema attr

p2 = p.name.  # complete builtin (str) function

p3 = subpkg.  # complete user module definition

import math
math.         # complete user module definition

"a".
