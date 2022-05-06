import base64 as _base64


def KMANGLED_encode(value: str, encoding: str = "utf-8") -> str:
    return _base64.b64encode(value.encode(encoding)).decode(encoding)


def KMANGLED_decode(value: str, encoding: str = "utf-8") -> str:
    return _base64.b64decode(value).decode(encoding)
