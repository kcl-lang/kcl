from typing import Callable, Optional, Dict, Set
from functools import wraps
import inspect

from kclvm.api.object import (
    KCLObject,
    KCLNoneObject,
    KCLTrueObject,
    KCLSchemaObject,
    KCLBuiltinFunctionObject,
    to_kcl_obj,
)
from kclvm.api.object.internal import kcl_option
from kclvm.compiler.build.utils import units

import kclvm.kcl.info as kcl_info

STANDARD_SYSTEM_MODULES = [
    "collection",
    "net",
    "math",
    "datetime",
    "regex",
    "yaml",
    "json",
    "crypto",
    "base64",
    "testing",
    "units",
]
STANDARD_SYSTEM_MODULE_LOCATION = "kclvm.compiler.extension.builtin.system_module"


def kcl_builtin(func):
    """KCL builtin decorator"""

    @wraps(func)
    def decorated(*args, **kwargs):
        return func(*args, **kwargs)

    return decorated


def kcl_obj_builtin(func):
    """KCL builtin decorator"""

    @wraps(func)
    def decorated(*args, **kwargs):
        return func(*args, **kwargs)

    return decorated


@kcl_builtin
def KMANGLED_option(
    key: str, *, type="", required=False, default=None, help="", file="", line=0
):
    """Return the top level argument by the key"""

    return kcl_option(
        key,
        type=type,
        required=required,
        default=default,
        help=help,
        file=file,
        line=line,
    )


@kcl_builtin
def KMANGLED_print(*args, **kwargs):
    """Prints the values to stdout"""
    import builtins

    builtins.print(*args, **kwargs)


@kcl_builtin
def KMANGLED_multiplyof(a, b):
    """Check if the modular result of a and b is 0"""
    if isinstance(a, int) and isinstance(b, int):
        return (a % b) == 0
    return False


@kcl_builtin
def KMANGLED_isunique(inval):
    """Check if a list has duplicated elements"""
    if isinstance(inval, list):
        if len(inval) == len(set(inval)):
            return True
    return False


@kcl_builtin
def KMANGLED_len(inval):
    """Return the length of a value"""
    import builtins

    return builtins.len(inval)


@kcl_builtin
def KMANGLED_abs(*args, **kwargs):
    """Return the absolute value of the argument."""
    import builtins

    return builtins.abs(*args, **kwargs)


@kcl_builtin
def KMANGLED_all_true(*args, **kwargs):
    """Return True if bool(x) is True for all values x in the iterable.

    If the iterable is empty, return True.
    """
    import builtins

    return builtins.all(*args, **kwargs)


@kcl_builtin
def KMANGLED_any_true(*args, **kwargs):
    """Return True if bool(x) is True for any x in the iterable.

    If the iterable is empty, return False.
    """
    import builtins

    return builtins.any(*args, **kwargs)


@kcl_builtin
def KMANGLED_hex(*args, **kwargs):
    """Return the hexadecimal representation of an integer."""
    import builtins

    return builtins.hex(*args, **kwargs)


@kcl_builtin
def KMANGLED_sorted(*args, **kwargs):
    """Return a new list containing all items from the iterable in ascending order.

    A custom key function can be supplied to customize the sort order, and the reverse
    flag can be set to request the result in descending order.
    """
    import builtins

    return [x for x in builtins.sorted(*args, **kwargs)]


@kcl_builtin
def KMANGLED_bin(*args, **kwargs):
    """Return the binary representation of an integer."""
    import builtins

    return builtins.bin(*args, **kwargs)


@kcl_builtin
def KMANGLED_oct(*args, **kwargs):
    """Return the octal representation of an integer."""
    import builtins

    return builtins.oct(*args, **kwargs)


@kcl_builtin
def KMANGLED_ord(*args, **kwargs):
    """Return the Unicode code point for a one-character string."""
    import builtins

    return builtins.ord(*args, **kwargs)


@kcl_builtin
def KMANGLED_range(*args, **kwargs):
    """Return the range of a value"""
    import builtins

    return [x for x in builtins.range(*args, **kwargs)]


@kcl_builtin
def KMANGLED_max(*args, **kwargs):
    """With a single iterable argument, return its biggest item.
    The default keyword-only argument specifies an object to return
    if the provided iterable is empty. With two or more arguments,
    return the largest argument.
    """
    import builtins

    return builtins.max(*args, **kwargs)


@kcl_builtin
def KMANGLED_min(*args, **kwargs):
    """With a single iterable argument, return its smallest item.
    The default keyword-only argument specifies an object to return
    if the provided iterable is empty. With two or more arguments,
    return the smallest argument.
    """
    import builtins

    return builtins.min(*args, **kwargs)


@kcl_builtin
def KMANGLED_sum(*args, **kwargs):
    """When the iterable is empty, return the start value. This function is
    intended specifically for use with numeric values and may reject
    non-numeric types.
    """
    import builtins

    return builtins.sum(*args, **kwargs)


@kcl_builtin
def KMANGLED_pow(*args, **kwargs):
    """Equivalent to x**y (with two arguments) or x**y % z (with three arguments)

    Some types, such as ints, are able to use a more efficient algorithm when
    invoked using the three argument form.
    """
    import builtins

    return builtins.pow(*args, **kwargs)


@kcl_builtin
def KMANGLED_round(*args, **kwargs):
    """Round a number to a given precision in decimal digits.

    The return value is an integer if ndigits is omitted or None.
    Otherwise the return value has the same type as the number.
    ndigits may be negative.
    """
    import builtins

    return builtins.round(*args, **kwargs)


@kcl_builtin
def KMANGLED_zip(*args, **kwargs):
    """Return a zip object whose .__next__() method returns
    a tuple where the i-th element comes from the i-th iterable
    argument. The .__next__() method continues until the shortest
    iterable in the argument sequence is exhausted and then
    it raises StopIteration.
    """
    import builtins

    return [list(r) for r in builtins.zip(*args, **kwargs)]


@kcl_builtin
def KMANGLED_int(*args, **kwargs) -> int:
    """Convert a number or string to an integer, or return 0 if no arguments
    are given. If x is a number, return x.__int__(). For floating point numbers,
    this truncates towards zero.
    """
    args = list(args)
    if len(args) >= 1 and isinstance(args[0], str):
        args[0] = str(units.to_quantity(args[0]))
    return int(*tuple(args), **kwargs)


@kcl_builtin
def KMANGLED_float(x) -> float:
    """Convert a string or number to a floating point number, if possible."""
    return float(x)


@kcl_builtin
def KMANGLED_str(x) -> str:
    """Create a new string object from the given object.
    If encoding or errors is specified, then the object must
    expose a data buffer that will be decoded using the
    given encoding and error handler.
    """
    return str(x)


@kcl_builtin
def KMANGLED_list(*args, **kwargs) -> list:
    """Built-in mutable sequence.

    If no argument is given, the constructor creates a new empty list.
    The argument must be an iterable if specified.
    """
    return list(*args, **kwargs)


@kcl_builtin
def KMANGLED_dict(x) -> dict:
    """dict() -> new empty dictionary dict(mapping) -> new dictionary initialized from a
    mapping object's (key, value) pairs dict(iterable) -> new dictionary initialized
    as if via: d = {} for k, v in iterable: d[k] = v dict(**kwargs) -> new dictionary
    initialized with the name=value pairs in the keyword argument list.
    For example: dict(one=1, two=2)
    """
    return dict(x)


@kcl_builtin
def KMANGLED_bool(x) -> bool:
    """Returns True when the argument x is true, False otherwise.
    The builtins True and False are the only two instances of the class bool.
    The class bool is a subclass of the class int, and cannot be subclassed.
    """
    return bool(x)


@kcl_obj_builtin
def KMANGLED_typeof(x: any, *, full_name: bool = False) -> str:
    """Return the type of the kcl object"""
    if isinstance(full_name, KCLTrueObject):
        full_name = True

    if x is None or isinstance(x, KCLNoneObject):
        return "None"

    if isinstance(x, KCLSchemaObject):
        if full_name:
            return x.full_type_str()
        else:
            return x.type_str()

    if isinstance(x, KCLObject):
        return x.type_str()
    else:
        return type(x).__name__


BUILTIN_FUNCTIONS_MAP = {
    "option": KMANGLED_option,
    "print": KMANGLED_print,
    "multiplyof": KMANGLED_multiplyof,
    "isunique": KMANGLED_isunique,
    "len": KMANGLED_len,
    "abs": KMANGLED_abs,
    "all_true": KMANGLED_all_true,
    "any_true": KMANGLED_any_true,
    "hex": KMANGLED_hex,
    "sorted": KMANGLED_sorted,
    "bin": KMANGLED_bin,
    "oct": KMANGLED_oct,
    "ord": KMANGLED_ord,
    "range": KMANGLED_range,
    "max": KMANGLED_max,
    "min": KMANGLED_min,
    "sum": KMANGLED_sum,
    "pow": KMANGLED_pow,
    "round": KMANGLED_round,
    "zip": KMANGLED_zip,
    "bool": KMANGLED_bool,
    "int": KMANGLED_int,
    "str": KMANGLED_str,
    "float": KMANGLED_float,
    "list": KMANGLED_list,
    "dict": KMANGLED_dict,
    "typeof": KMANGLED_typeof,
}

BUILTIN_FUNCTIONS = list(BUILTIN_FUNCTIONS_MAP.keys())


def get_builtin_func_objects():
    """Get all builtin function objects"""
    return [
        KCLBuiltinFunctionObject(name=builtin, function=BUILTIN_FUNCTIONS_MAP[builtin])
        for builtin in BUILTIN_FUNCTIONS
    ]


def new_builtin_function(
    name: str, func: Callable
) -> Optional[KCLBuiltinFunctionObject]:
    """New a KCL builtin function object using native python function"""
    if not func or not name:
        return None
    return KCLBuiltinFunctionObject(name=name, function=func)


def get_system_module_func_objects(
    module_name: str,
) -> Dict[str, KCLBuiltinFunctionObject]:
    """Get all KCL builtin functions from the standard system module named 'module_name'"""
    if not module_name or module_name not in STANDARD_SYSTEM_MODULES:
        return {}
    module = __import__(
        f"{STANDARD_SYSTEM_MODULE_LOCATION}.{module_name}",
        fromlist=STANDARD_SYSTEM_MODULE_LOCATION,
    )
    members = inspect.getmembers(module)
    result = {
        kcl_info.demangle(member_name): new_builtin_function(
            kcl_info.demangle(member_name), member
        )
        if inspect.isfunction(member)
        else to_kcl_obj(member)
        for member_name, member in members
        if kcl_info.ismangled(member_name)
    }
    return result


def get_system_module_members(
    module_name: str,
) -> Dict[str, Set[str]]:
    """Get all members from the standard system module named 'module_name'"""
    if not module_name or module_name not in STANDARD_SYSTEM_MODULES:
        return {}
    module = __import__(
        f"{STANDARD_SYSTEM_MODULE_LOCATION}.{module_name}",
        fromlist=STANDARD_SYSTEM_MODULE_LOCATION,
    )
    members = inspect.getmembers(module)
    result = {
        kcl_info.demangle(member_name)
        for member_name, _ in members
        if kcl_info.ismangled(member_name)
    }
    return result
