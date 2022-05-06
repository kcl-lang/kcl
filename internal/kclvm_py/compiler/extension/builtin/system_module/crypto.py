import hashlib as _hashlib


def KMANGLED_md5(value: str, encoding: str = "utf-8") -> str:
    return _hashlib.md5(value.encode(encoding)).hexdigest()


def KMANGLED_sha1(value: str, encoding: str = "utf-8") -> str:
    return _hashlib.sha1(value.encode(encoding)).hexdigest()


def KMANGLED_sha224(value: str, encoding: str = "utf-8") -> str:
    return _hashlib.sha224(value.encode(encoding)).hexdigest()


def KMANGLED_sha256(value: str, encoding: str = "utf-8") -> str:
    return _hashlib.sha256(value.encode(encoding)).hexdigest()


def KMANGLED_sha384(value: str, encoding: str = "utf-8") -> str:
    return _hashlib.sha384(value.encode(encoding)).hexdigest()


def KMANGLED_sha512(value: str, encoding: str = "utf-8") -> str:
    return _hashlib.sha512(value.encode(encoding)).hexdigest()
