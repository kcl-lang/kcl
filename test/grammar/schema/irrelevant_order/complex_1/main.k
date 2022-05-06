
import math

schema Base:
    mixin [BaseMixin]
    base_templating: str = "base"

schema Sub[arg](Base):
    mixin [SubMixin]
    # print("the value of a is", a)
    a: int = b + c + e
    b: int = 2
    _c = 3
    _c = 2
    _c = 4
    c: int = _c
    e: int = _e
    f: int = _f
    d = 4
    if d:
        _e = 2
        _f = 3
    # print("the value of d is", d)
    sub_templating: str = "sub"

schema BaseMixin:
    base_mixin: str = "base"

schema SubMixin:
    sub_mixin: str = "sub"

data = Sub(1) {}
