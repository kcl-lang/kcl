# Copyright 2021 The KCL Authors. All rights reserved.

MANGLE_PREFIX = "KMANGLED_"
TAGGING_PREFIX = "KTAG_"


def mangle(name):
    """Mangle a name"""
    dot = name.rfind(".")
    if dot == -1:
        mangled_name = MANGLE_PREFIX + name
    else:
        mangled_name = mangle(name[:dot]) + "." + mangle(name[dot + 1 :])
    return mangled_name


def demangle(name):
    """Demangle a name if it is mangled"""
    demangled_name = ""
    if ismangled(name):
        dot = name.rfind(".")
        if dot == -1:
            assert len(name) > len(
                MANGLE_PREFIX
            ), "Internal Error: Demangling failure. Please report a bug to us."
            demangled_name += name[len(MANGLE_PREFIX) :]
        else:
            demangled_name = demangle(name[:dot]) + "." + demangle(name[dot + 1 :])
    else:
        demangled_name = name
    return demangled_name


def ismangled(name):
    """Check if a name is mangled"""
    if name.startswith(MANGLE_PREFIX):
        return True
    return False


def tagging(tag, name=None):
    """tagging a name"""
    return TAGGING_PREFIX + tag + "_" + name


def detagging(tag, name=None):
    """Detagging a name if it is tagged"""
    if istagged(name):
        return name[len(TAGGING_PREFIX) + len(tag) + 1 :]
    return name


def istagged(name):
    """Check if a name is tagged"""
    if name.startswith(TAGGING_PREFIX):
        return True
    return False


def isprivate_field(name):
    return name.startswith("_")
