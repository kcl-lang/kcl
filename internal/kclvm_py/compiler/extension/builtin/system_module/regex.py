#! /usr/bin/env python3
import re as _re


def KMANGLED_replace(string: str, pattern: str, replace: str, count: int = 0):
    """
    Return the string obtained by replacing the leftmost non-overlapping occurrences
    of the pattern in string by the replacement.
    """
    return _re.sub(pattern, replace, string, count)


def KMANGLED_match(string: str, pattern: str):
    """
    Try to apply the pattern at the start of the string, returning a Match object, or None if no match was found.
    """
    return bool(_re.match(pattern, string))


def KMANGLED_compile(pattern: str):
    """
    Compile a regular expression pattern, returning a bool value denoting whether the pattern is valid
    """
    return bool(_re.compile(pattern))


def KMANGLED_findall(string: str, pattern: str):
    """
    Return a list of all non-overlapping matches in the string.
    """
    return _re.findall(pattern, string)


def KMANGLED_search(string: str, pattern: str):
    """
    Scan through string looking for a match to the pattern,
    returning a Match object, or None if no match was found.
    """
    return bool(_re.search(pattern, string))


def KMANGLED_split(string: str, pattern: str, maxsplit: int = 0):
    """
    Scan through string looking for a match to the pattern,
    returning a Match object, or None if no match was found.
    """
    return _re.split(pattern, string, maxsplit)
