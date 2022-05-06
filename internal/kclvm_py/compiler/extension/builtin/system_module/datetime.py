#! /usr/bin/env python3

from datetime import datetime as dt
import time as _time


def KMANGLED_today():
    """
    Return the datetime today
    """
    return str(dt.today())


def KMANGLED_now():
    """
    Return the time at now
    """
    return _time.asctime(_time.localtime(_time.time()))


def KMANGLED_ticks() -> float:
    """
    Return the current time in seconds since the Epoch.
    """
    return _time.time()


def KMANGLED_date() -> str:
    """
    Return the datetime string
    """
    return _time.strftime("%Y-%m-%d %H:%M:%S", _time.localtime())
