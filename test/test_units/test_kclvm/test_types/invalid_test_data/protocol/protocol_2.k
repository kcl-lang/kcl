import pkg

schema DataProtocol:
    data: str

schema DataMixin for pkg.DataProtocol:
    x: int = data
