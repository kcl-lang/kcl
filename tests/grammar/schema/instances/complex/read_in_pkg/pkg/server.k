schema Server:
    name: str

schema Check:
    check:
        len(Server.instances(full_pkg=True)) == 2, "lost Server instances"

names = lambda {
    [s.name for s in Server.instances(full_pkg=True)]
}
