import .pkg
import .pkg.subpkg as sub

schema Person2:
    a: bool
    b: "abc" | "def"
    c: [int]
    d: 1
    e: True
    f: {str:str}
    g: sub.Person1

p: Person2{
    a  # complete `True` and `False`

    b  # complete `"abc"` and `"def"`

    c  # complete `[]`

    d  # complete `1`

    e  # complete `True`

    f  # complete `{}`

    g  # complete `subpkg.Person1{}`
}
