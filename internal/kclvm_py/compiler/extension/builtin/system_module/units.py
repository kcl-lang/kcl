# Copyright 2020 The KCL Authors. All rights reserved.

from typing import Union

import kclvm.api.object as objpkg

# --------------------------------------
# Numerical Constants
# Usage:
#     import units
#     memory = 1024 * units.Mi  # 1024Mi
# --------------------------------------

KMANGLED_n = 1e-09
KMANGLED_u = 1e-06
KMANGLED_m = 0.001
KMANGLED_k = 1_000
KMANGLED_K = 1_000
KMANGLED_M = 1_000_000
KMANGLED_G = 1_000_000_000
KMANGLED_T = 1_000_000_000_000
KMANGLED_P = 1_000_000_000_000_000
KMANGLED_Ki = 1024
KMANGLED_Mi = 1024 ** 2
KMANGLED_Gi = 1024 ** 3
KMANGLED_Ti = 1024 ** 4
KMANGLED_Pi = 1024 ** 5

UNIT_MAPPING = {
    "n": KMANGLED_n,
    "u": KMANGLED_u,
    "m": KMANGLED_m,
    "k": KMANGLED_k,
    "K": KMANGLED_K,
    "M": KMANGLED_M,
    "G": KMANGLED_G,
    "T": KMANGLED_T,
    "P": KMANGLED_P,
    "Ki": KMANGLED_Ki,
    "Mi": KMANGLED_Mi,
    "Gi": KMANGLED_Gi,
    "Ti": KMANGLED_Ti,
    "Pi": KMANGLED_Pi,
}

# --------------------------------------
# Numerical Multiplier Type
# Usage:
#     import units
#     memory: units.NumberMultiplier = 1M
# --------------------------------------

KMANGLED_NumberMultiplier = objpkg.KCLNumberMultiplierTypeObject()

# ------------------------------------------
# Unit ToString Methods
# Usage:
#     import units
#     disk = units.to_Ki(1024)  # "1Ki"
# Input:
#     num: int
# Returns:
#     int
# Raises:
#     ValueError on invalid or unknown input
# ------------------------------------------


def KMANGLED_to_n(num: int) -> str:
    """Int literal to string with `n` suffix"""
    return to_unit(num, "n")


def KMANGLED_to_u(num: int) -> str:
    """Int literal to string with `u` suffix"""
    return to_unit(num, "u")


def KMANGLED_to_m(num: int) -> str:
    """Int literal to string with `m` suffix"""
    return to_unit(num, "m")


def KMANGLED_to_K(num: int) -> str:
    """Int literal to string with `K` suffix"""
    return to_unit(num, "K")


def KMANGLED_to_M(num: int) -> str:
    """Int literal to string with `M` suffix"""
    return to_unit(num, "M")


def KMANGLED_to_G(num: int) -> str:
    """Int literal to string with `G` suffix"""
    return to_unit(num, "G")


def KMANGLED_to_T(num: int) -> str:
    """Int literal to string with `T` suffix"""
    return to_unit(num, "T")


def KMANGLED_to_P(num: int) -> str:
    """Int literal to string with `P` suffix"""
    return to_unit(num, "P")


def KMANGLED_to_Ki(num: int) -> str:
    """Int literal to string with `Ki` suffix"""
    return to_unit(num, "Ki")


def KMANGLED_to_Mi(num: int) -> str:
    """Int literal to string with `Mi` suffix"""
    return to_unit(num, "Mi")


def KMANGLED_to_Gi(num: int) -> str:
    """Int literal to string with `Gi` suffix"""
    return to_unit(num, "Gi")


def KMANGLED_to_Ti(num: int) -> str:
    """Int literal to string with `Ti` suffix"""
    return to_unit(num, "Ti")


def KMANGLED_to_Pi(num: int) -> str:
    """Int literal to string with `Pi` suffix"""
    return to_unit(num, "Pi")


def to_unit(num: Union[int, float], suffix: str) -> str:
    """Connect numbers and suffixes"""
    if not isinstance(num, (int, float)):
        raise ValueError("Unsupported number type: {}".format(type(num)))
    if not suffix or not isinstance(suffix, str) or suffix not in list(UNIT_MAPPING):
        raise ValueError("Unsupported unit suffix: {}".format(suffix))
    return str(int(num // UNIT_MAPPING[suffix])) + suffix
