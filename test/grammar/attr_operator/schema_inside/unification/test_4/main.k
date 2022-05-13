schema Config:
    [str]: str

_c = Config {}
_c |= {
    key = "value"
}
c = _c
