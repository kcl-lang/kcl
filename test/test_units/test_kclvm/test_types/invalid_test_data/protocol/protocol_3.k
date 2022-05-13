import pkg

schema DataProtocol:
    data: str

schema DataMixin for pkg.pkg.DataProtocol:
    x: int = data
