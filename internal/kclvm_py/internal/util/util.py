import hashlib
import typing


class dotdict(dict):
    """dot.notation access to dictionary attributes"""

    __getattr__ = dict.get
    __setattr__ = dict.__setitem__
    __delattr__ = dict.__delitem__


def hash(input_str):
    return hashlib.md5(input_str.encode("utf-8")).hexdigest()


def merge_option_same_keys(args):
    """
    merge kcl -D and -Y argument with the same keys
    """
    if not args:
        return {}
    return {k: v for k, v in args}


def safe_call(fn: typing.Callable, *args, **kwargs) -> (typing.Any, Exception):
    result = None
    try:
        result = fn(*args, **kwargs)
        return result, None
    except Exception as err:
        return result, err
