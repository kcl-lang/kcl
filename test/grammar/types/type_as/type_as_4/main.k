import pkg

schema Sub(pkg.Base):
    data?: int = 2

base: pkg.Base = pkg.Base {} as pkg.Base
sub: Sub = Sub {} as Sub
