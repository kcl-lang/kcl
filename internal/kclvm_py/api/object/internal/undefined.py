# Copyright 2021 The KCL Authors. All rights reserved.


class UndefinedType:
    def __repr__(self):
        return self.__str__()

    def __str__(self):
        """Built-in str(Undefined) like str(None)"""
        return "Undefined"

    def __bool__(self):
        """Built-in bool(Undefined) like bool(None)"""
        return False

    @staticmethod
    def type_str():
        """Error message show type"""
        return "UndefinedType"

    @property
    def value(self):
        return self._value

    def __init__(self):
        self._value = None


Undefined = UndefinedType()
