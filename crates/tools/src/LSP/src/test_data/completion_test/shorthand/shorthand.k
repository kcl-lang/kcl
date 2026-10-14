schema Server:
    debug: bool
    verbose: bool

schema Cluster:
    server: Server

debug = True

s1 = Server {
    ver
    debug: True
}

s2 = Server {
    debug
}

c = Cluster {
    server: {
        verbose
    }
}

c2 = Cluster {
    server: {
        ver
    }
}

schema Types:
    a: bool
    b: "abc" | "def"
    c: [int]
    d: 1
    e: True
    f: {str:str}
    g: Server

t = Types {
    a
    b
    c
    d
    e
    f
    g
}
