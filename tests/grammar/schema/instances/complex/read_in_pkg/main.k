import .pkg

servers = [pkg.Server {name = n} for n in ["a", "b"]]
check = pkg.Check {}
names = pkg.names()
