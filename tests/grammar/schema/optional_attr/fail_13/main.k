schema S:
    a: int
    b: str

schema L:
    ss?: [S]
    sss?: {str:S}

L {
    ss = [S {b = "b"}]
}
