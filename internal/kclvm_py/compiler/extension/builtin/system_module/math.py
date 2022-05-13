#! /usr/bin/env python3

import math


def KMANGLED_ceil(*args, **kwargs):
    return math.ceil(*args, **kwargs)


def KMANGLED_factorial(*args, **kwargs):
    return math.factorial(*args, **kwargs)


def KMANGLED_floor(*args, **kwargs):
    return math.floor(*args, **kwargs)


def KMANGLED_gcd(*args, **kwargs):
    return math.gcd(*args, **kwargs)


def KMANGLED_isfinite(*args, **kwargs):
    return math.isfinite(*args, **kwargs)


def KMANGLED_isinf(*args, **kwargs):
    return math.isinf(*args, **kwargs)


def KMANGLED_isnan(*args, **kwargs):
    return math.isnan(*args, **kwargs)


def KMANGLED_modf(x):
    return list(math.modf(x))


def KMANGLED_exp(*args, **kwargs):
    return math.exp(*args, **kwargs)


def KMANGLED_expm1(*args, **kwargs):
    return math.expm1(*args, **kwargs)


def KMANGLED_log(*args, **kwargs):
    return math.log(*args, **kwargs)


def KMANGLED_log1p(*args, **kwargs):
    return math.log1p(*args, **kwargs)


def KMANGLED_log2(*args, **kwargs):
    return math.log2(*args, **kwargs)


def KMANGLED_log10(n):
    return math.log10(n)


def KMANGLED_pow(*args, **kwargs):
    return math.pow(*args, **kwargs)


def KMANGLED_sqrt(*args, **kwargs):
    return math.sqrt(*args, **kwargs)
