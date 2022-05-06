import pkg

a = 1

schema DataProtocol:
    data: str

schema DataMixin for pkg.pkg.DataProtocol:
    x: str = data

schema AMixin for a:
    x: str = data

schema Data(pkg.pkg.Base) for DataProtocol:
    x: str = data

schema A(a) for a:
    x: str = data
