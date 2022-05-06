schema S:
    [str]: []

_s: S {
    a += [1]
}
_s |= {
    b += [1]
}
s = _s
