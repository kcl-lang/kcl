protocol AProtocol:
    name?: str

mixin AMixin for AProtocol:
    config = {
        data = name
    }
    data = name