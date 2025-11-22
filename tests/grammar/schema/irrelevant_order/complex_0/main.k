import math

schema Base:
    mixin [BaseMixin]
    base_templating: str = "base"

schema Sub[arg](Base):
    mixin [SubMixin]
    a: int = b + c + e
    b: int = 2
    c: int = 3
    e: int
    f: int
    c = 2
    c = 4
    d = 4
    if d:
        e = 2
        f = 3
    sub_templating: str = "sub"

schema BaseMixin:
    base_mixin: str = "base"

schema SubMixin:
    sub_mixin: str = "sub"

data = Sub(1) {}
