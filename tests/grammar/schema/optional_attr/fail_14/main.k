schema S:
    a: int
    b: str

schema L:
    ss?: [S]
    sss?: {str:S}

L {
    sss.aa: S {b = "2"}
}
