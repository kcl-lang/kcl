# Copyright 2021 The KCL Authors. All rights reserved.

import sys
import typing
import unittest
import math as pymath
import struct

import kclvm_runtime

# https://github.com/python/cpython/blob/main/Lib/test/test_math.py


_Dylib = kclvm_runtime.KclvmRuntimeDylib()


eps = 1e-05
NAN = float("nan")
INF = float("inf")
NINF = float("-inf")
FLOAT_MAX = sys.float_info.max
FLOAT_MIN = sys.float_info.min


class kclx_Math:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib

    def ceil(self, x) -> int:
        return self.dylib.Invoke(f"math.ceil", x)

    def factorial(self, x) -> int:
        return self.dylib.Invoke(f"math.factorial", x)

    def floor(self, x) -> int:
        return self.dylib.Invoke(f"math.floor", x)

    def gcd(self, a: int, b: int) -> int:
        return self.dylib.Invoke(f"math.gcd", a, b)

    def isfinite(self, x) -> bool:
        return self.dylib.Invoke(f"math.isfinite", x)

    def isinf(self, x) -> bool:
        return self.dylib.Invoke(f"math.isinf", x)

    def isnan(self, x) -> bool:
        return self.dylib.Invoke(f"math.isnan", x)

    def modf(self, x) -> typing.Tuple[float, float]:
        return self.dylib.Invoke(f"math.modf", x)

    def exp(self, x) -> float:
        return self.dylib.Invoke(f"math.exp", x)

    def expm1(self, x) -> float:
        return self.dylib.Invoke(f"math.expm1", x)

    def log(self, x) -> float:
        return self.dylib.Invoke(f"math.log", x)

    def log1p(self, x) -> float:
        return self.dylib.Invoke(f"math.log1p", x)

    def log2(self, x) -> float:
        return self.dylib.Invoke(f"math.log2", x)

    def log10(self, x) -> float:
        return self.dylib.Invoke(f"math.log10", x)

    def pow(self, x, y) -> float:
        return self.dylib.Invoke(f"math.pow", x, y)

    def sqrt(self, x) -> float:
        return self.dylib.Invoke(f"math.sqrt", x)


math = kclx_Math()


def to_ulps(x):
    """Convert a non-NaN float x to an integer, in such a way that
    adjacent floats are converted to adjacent integers.  Then
    abs(ulps(x) - ulps(y)) gives the difference in ulps between two
    floats.
    The results from this function will only make sense on platforms
    where native doubles are represented in IEEE 754 binary64 format.
    Note: 0.0 and -0.0 are converted to 0 and -1, respectively.
    """
    n = struct.unpack("<q", struct.pack("<d", x))[0]
    if n < 0:
        n = ~(n + 2 ** 63)
    return n


def count_set_bits(n):
    """Number of '1' bits in binary expansion of a nonnnegative integer."""
    return 1 + count_set_bits(n & n - 1) if n else 0


def partial_product(start, stop):
    """Product of integers in range(start, stop, 2), computed recursively.
    start and stop should both be odd, with start <= stop.
    """
    numfactors = (stop - start) >> 1
    if not numfactors:
        return 1
    elif numfactors == 1:
        return start
    else:
        mid = (start + numfactors) | 1
        return partial_product(start, mid) * partial_product(mid, stop)


def py_factorial(n):
    """Factorial of nonnegative integer n, via "Binary Split Factorial Formula"
    described at http://www.luschny.de/math/factorial/binarysplitfact.html
    """
    inner = outer = 1
    for i in reversed(range(n.bit_length())):
        inner *= partial_product((n >> i + 1) + 1 | 1, (n >> i) + 1 | 1)
        outer *= inner
    return outer << (n - count_set_bits(n))


def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    # Common logic of MathTests.(ftest, test_testcases, test_mtestcases)
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).
    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.
    Returns None on success and an error message on failure.
    """

    # Check exactly equal (applies also to strings representing exceptions)
    if got == expected:
        return None

    failure = "not equal"

    # Turn mixed float and int comparison (e.g. floor()) to all-float
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)

    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            # Pass, since both nan
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            # We already know they're not equal, drop through to failure
            pass
        else:
            # Both are finite floats (now). Are they close enough?
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)

    # arguments are not equal, and if numeric, are too far apart
    if failure is not None:
        fail_fmt = "expected {!r}, got {!r}"
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += " ({})".format(failure)
        return fail_msg
    else:
        return None


def ulp_abs_check(expected, got, ulp_tol, abs_tol):
    """Given finite floats `expected` and `got`, check that they're
    approximately equal to within the given number of ulps or the
    given absolute tolerance, whichever is bigger.
    Returns None on success and an error message on failure.
    """
    ulp_error = abs(to_ulps(expected) - to_ulps(got))
    abs_error = abs(expected - got)

    # Succeed if either abs_error <= abs_tol or ulp_error <= ulp_tol.
    if abs_error <= abs_tol or ulp_error <= ulp_tol:
        return None
    else:
        fmt = "error = {:.3g} ({:d} ulps); " "permitted error = {:.3g} or {:d} ulps"
        return fmt.format(abs_error, ulp_error, abs_tol, ulp_tol)


def result_check(expected, got, ulp_tol=5, abs_tol=0.0):
    # Common logic of MathTests.(ftest, test_testcases, test_mtestcases)
    """Compare arguments expected and got, as floats, if either
    is a float, using a tolerance expressed in multiples of
    ulp(expected) or absolutely (if given and greater).
    As a convenience, when neither argument is a float, and for
    non-finite floats, exact equality is demanded. Also, nan==nan
    as far as this function is concerned.
    Returns None on success and an error message on failure.
    """

    # Check exactly equal (applies also to strings representing exceptions)
    if got == expected:
        return None

    failure = "not equal"

    # Turn mixed float and int comparison (e.g. floor()) to all-float
    if isinstance(expected, float) and isinstance(got, int):
        got = float(got)
    elif isinstance(got, float) and isinstance(expected, int):
        expected = float(expected)

    if isinstance(expected, float) and isinstance(got, float):
        if math.isnan(expected) and math.isnan(got):
            # Pass, since both nan
            failure = None
        elif math.isinf(expected) or math.isinf(got):
            # We already know they're not equal, drop through to failure
            pass
        else:
            # Both are finite floats (now). Are they close enough?
            failure = ulp_abs_check(expected, got, ulp_tol, abs_tol)

    # arguments are not equal, and if numeric, are too far apart
    if failure is not None:
        fail_fmt = "expected {!r}, got {!r}"
        fail_msg = fail_fmt.format(expected, got)
        fail_msg += " ({})".format(failure)
        return fail_msg
    else:
        return None


class BaseTest(unittest.TestCase):
    def ftest(self, name, got, expected, ulp_tol=5, abs_tol=0.0):
        """Compare arguments expected and got, as floats, if either
        is a float, using a tolerance expressed in multiples of
        ulp(expected) or absolutely, whichever is greater.
        As a convenience, when neither argument is a float, and for
        non-finite floats, exact equality is demanded. Also, nan==nan
        in this function.
        """
        failure = result_check(expected, got, ulp_tol, abs_tol)
        if failure is not None:
            self.fail("{}: {}".format(name, failure))

    def testCeil(self):
        # self.assertRaises(TypeError, math.ceil)
        self.assertEqual(int, type(math.ceil(0.5)))
        self.assertEqual(math.ceil(0.5), 1)
        self.assertEqual(math.ceil(1.0), 1)
        self.assertEqual(math.ceil(1.5), 2)
        self.assertEqual(math.ceil(-0.5), 0)
        self.assertEqual(math.ceil(-1.0), -1)
        self.assertEqual(math.ceil(-1.5), -1)
        self.assertEqual(math.ceil(0.0), 0)
        self.assertEqual(math.ceil(-0.0), 0)

        # self.assertEqual(math.ceil(INF), INF)
        # self.assertEqual(math.ceil(NINF), NINF)
        # self.assertTrue(math.isnan(math.ceil(NAN)))

    def testExp(self):
        # self.assertRaises(TypeError, math.exp)
        self.ftest("exp(-1)", math.exp(-1), 1 / pymath.e)
        self.ftest("exp(0)", math.exp(0), 1)
        self.ftest("exp(1)", math.exp(1), pymath.e)
        # self.assertEqual(math.exp(INF), INF)
        # self.assertEqual(math.exp(NINF), 0.0)
        # self.assertTrue(math.isnan(math.exp(NAN)))
        # self.assertRaises(OverflowError, math.exp, 1000000)

    def testFactorial(self):
        self.assertEqual(math.factorial(0), 1)
        total = 1
        for i in range(1, 20):
            total *= i
            self.assertEqual(math.factorial(i), total)
            self.assertEqual(math.factorial(i), py_factorial(i))
        # self.assertRaises(ValueError, math.factorial, -1)
        # self.assertRaises(ValueError, math.factorial, -10**100)

    def testFloor(self):
        self.assertEqual(math.floor(0.5), 0)
        self.assertEqual(math.floor(1.0), 1)
        self.assertEqual(math.floor(1.5), 1)
        self.assertEqual(math.floor(-0.5), -1)
        self.assertEqual(math.floor(-1.0), -1)
        self.assertEqual(math.floor(-1.5), -2)
        # self.assertEqual(math.ceil(INF), INF)
        # self.assertEqual(math.ceil(NINF), NINF)
        # self.assertTrue(math.isnan(math.floor(NAN)))

    def testGcd(self):
        gcd = math.gcd
        self.assertEqual(gcd(0, 0), 0)
        self.assertEqual(gcd(1, 0), 1)
        self.assertEqual(gcd(-1, 0), 1)
        self.assertEqual(gcd(0, 1), 1)
        self.assertEqual(gcd(0, -1), 1)
        self.assertEqual(gcd(7, 1), 1)
        self.assertEqual(gcd(7, -1), 1)
        self.assertEqual(gcd(-23, 15), 1)
        self.assertEqual(gcd(120, 84), 12)
        self.assertEqual(gcd(84, -120), 12)

    def testLog(self):
        # self.assertRaises(TypeError, math.log)
        self.ftest("log(1/e)", math.log(1 / pymath.e), -1)
        self.ftest("log(1)", math.log(1), 0)
        self.ftest("log(e)", math.log(pymath.e), 1)
        # self.ftest("log(32,2)", math.log(32, 2), 5)
        # self.ftest("log(10**40, 10)", math.log(10 ** 40, 10), 40)
        # self.ftest("log(10**40, 10**20)", math.log(10 ** 40, 10 ** 20), 2)
        # self.ftest("log(10**1000)", math.log(10 ** 1000), 2302.5850929940457)
        # self.assertRaises(ValueError, math.log, -1.5)
        # self.assertRaises(ValueError, math.log, -10**1000)
        # self.assertRaises(ValueError, math.log, NINF)
        # self.assertEqual(math.log(INF), INF)
        # self.assertTrue(math.isnan(math.log(NAN)))

    def testLog2(self):
        # self.assertRaises(TypeError, math.log2)

        # Check some integer values
        self.assertEqual(math.log2(1), 0.0)
        self.assertEqual(math.log2(2), 1.0)
        self.assertEqual(math.log2(4), 2.0)

        # Large integer values
        # self.assertEqual(math.log2(2**1023), 1023.0)
        # self.assertEqual(math.log2(2**1024), 1024.0)
        # self.assertEqual(math.log2(2**2000), 2000.0)

        # self.assertRaises(ValueError, math.log2, -1.5)
        # self.assertRaises(ValueError, math.log2, NINF)
        # self.assertTrue(math.isnan(math.log2(NAN)))

    def testPow(self):
        # self.assertRaises(TypeError, math.pow)
        self.ftest("pow(0,1)", math.pow(0, 1), 0)
        self.ftest("pow(1,0)", math.pow(1, 0), 1)
        self.ftest("pow(2,1)", math.pow(2, 1), 2)
        self.ftest("pow(2,-1)", math.pow(2, -1), 0.5)
        # self.assertEqual(math.pow(INF, 1), INF)
        # self.assertEqual(math.pow(NINF, 1), NINF)
        # self.assertEqual((math.pow(1, INF)), 1.0)
        # self.assertEqual((math.pow(1, NINF)), 1.0)
        # self.assertTrue(math.isnan(math.pow(NAN, 1)))
        # self.assertTrue(math.isnan(math.pow(2, NAN)))
        # self.assertTrue(math.isnan(math.pow(0, NAN)))
        # self.assertEqual(math.pow(1, NAN), 1)

        # pow(0., x)
        # self.assertEqual(math.pow(0.0, INF), 0.0)
        self.assertEqual(math.pow(0.0, 3.0), 0.0)
        self.assertEqual(math.pow(0.0, 2.3), 0.0)
        self.assertEqual(math.pow(0.0, 2.0), 0.0)
        self.assertEqual(math.pow(0.0, 0.0), 1.0)
        self.assertEqual(math.pow(0.0, -0.0), 1.0)
        # self.assertRaises(ValueError, math.pow, 0.0, -2.0)
        # self.assertRaises(ValueError, math.pow, 0.0, -2.3)
        # self.assertRaises(ValueError, math.pow, 0.0, -3.0)
        # self.assertEqual(math.pow(0.0, NINF), INF)
        # self.assertTrue(math.isnan(math.pow(0.0, NAN)))

        # pow(INF, x)
        # self.assertEqual(math.pow(INF, INF), INF)
        # self.assertEqual(math.pow(INF, 3.0), INF)
        # self.assertEqual(math.pow(INF, 2.3), INF)
        # self.assertEqual(math.pow(INF, 2.0), INF)
        # self.assertEqual(math.pow(INF, 0.0), 1.0)
        # self.assertEqual(math.pow(INF, -0.0), 1.0)
        # self.assertEqual(math.pow(INF, -2.0), 0.0)
        # self.assertEqual(math.pow(INF, -2.3), 0.0)
        # self.assertEqual(math.pow(INF, -3.0), 0.0)
        # self.assertEqual(math.pow(INF, NINF), 0.0)
        # self.assertTrue(math.isnan(math.pow(INF, NAN)))

        # pow(-0., x)
        # self.assertEqual(math.pow(-0.0, INF), 0.0)
        self.assertEqual(math.pow(-0.0, 3.0), -0.0)
        self.assertEqual(math.pow(-0.0, 2.3), 0.0)
        self.assertEqual(math.pow(-0.0, 2.0), 0.0)
        self.assertEqual(math.pow(-0.0, 0.0), 1.0)
        self.assertEqual(math.pow(-0.0, -0.0), 1.0)
        # self.assertRaises(ValueError, math.pow, -0.0, -2.0)
        # self.assertRaises(ValueError, math.pow, -0.0, -2.3)
        # self.assertRaises(ValueError, math.pow, -0.0, -3.0)
        # self.assertEqual(math.pow(-0.0, NINF), INF)
        # self.assertTrue(math.isnan(math.pow(-0.0, NAN)))

        # pow(NINF, x)
        # self.assertEqual(math.pow(NINF, INF), INF)
        # self.assertEqual(math.pow(NINF, 3.0), NINF)
        # self.assertEqual(math.pow(NINF, 2.3), INF)
        # self.assertEqual(math.pow(NINF, 2.0), INF)
        # self.assertEqual(math.pow(NINF, 0.0), 1.0)
        # self.assertEqual(math.pow(NINF, -0.0), 1.0)
        # self.assertEqual(math.pow(NINF, -2.0), 0.0)
        # self.assertEqual(math.pow(NINF, -2.3), 0.0)
        # self.assertEqual(math.pow(NINF, -3.0), -0.0)
        # self.assertEqual(math.pow(NINF, NINF), 0.0)
        # self.assertTrue(math.isnan(math.pow(NINF, NAN)))

        # pow(-1, x)
        # self.assertEqual(math.pow(-1.0, INF), 1.0)
        self.assertEqual(math.pow(-1.0, 3.0), -1.0)
        # self.assertRaises(ValueError, math.pow, -1.0, 2.3)
        self.assertEqual(math.pow(-1.0, 2.0), 1.0)
        self.assertEqual(math.pow(-1.0, 0.0), 1.0)
        self.assertEqual(math.pow(-1.0, -0.0), 1.0)
        self.assertEqual(math.pow(-1.0, -2.0), 1.0)
        # self.assertRaises(ValueError, math.pow, -1.0, -2.3)
        self.assertEqual(math.pow(-1.0, -3.0), -1.0)
        # self.assertEqual(math.pow(-1.0, NINF), 1.0)
        # self.assertTrue(math.isnan(math.pow(-1.0, NAN)))

        # pow(1, x)
        # self.assertEqual(math.pow(1.0, INF), 1.0)
        self.assertEqual(math.pow(1.0, 3.0), 1.0)
        self.assertEqual(math.pow(1.0, 2.3), 1.0)
        self.assertEqual(math.pow(1.0, 2.0), 1.0)
        self.assertEqual(math.pow(1.0, 0.0), 1.0)
        self.assertEqual(math.pow(1.0, -0.0), 1.0)
        self.assertEqual(math.pow(1.0, -2.0), 1.0)
        self.assertEqual(math.pow(1.0, -2.3), 1.0)
        self.assertEqual(math.pow(1.0, -3.0), 1.0)
        # self.assertEqual(math.pow(1.0, NINF), 1.0)
        # self.assertEqual(math.pow(1.0, NAN), 1.0)

        # pow(x, 0) should be 1 for any x
        self.assertEqual(math.pow(2.3, 0.0), 1.0)
        self.assertEqual(math.pow(-2.3, 0.0), 1.0)
        # self.assertEqual(math.pow(NAN, 0.0), 1.0)
        self.assertEqual(math.pow(2.3, -0.0), 1.0)
        self.assertEqual(math.pow(-2.3, -0.0), 1.0)
        # self.assertEqual(math.pow(NAN, -0.0), 1.0)

        # pow(x, y) is invalid if x is negative and y is not integral
        # self.assertRaises(ValueError, math.pow, -1.0, 2.3)
        # self.assertRaises(ValueError, math.pow, -15.0, -3.1)

        # pow(x, NINF)
        # self.assertEqual(math.pow(1.9, NINF), 0.0)
        # self.assertEqual(math.pow(1.1, NINF), 0.0)
        # self.assertEqual(math.pow(0.9, NINF), INF)
        # self.assertEqual(math.pow(0.1, NINF), INF)
        # self.assertEqual(math.pow(-0.1, NINF), INF)
        # self.assertEqual(math.pow(-0.9, NINF), INF)
        # self.assertEqual(math.pow(-1.1, NINF), 0.0)
        # self.assertEqual(math.pow(-1.9, NINF), 0.0)

        # pow(x, INF)
        # self.assertEqual(math.pow(1.9, INF), INF)
        # self.assertEqual(math.pow(1.1, INF), INF)
        # self.assertEqual(math.pow(0.9, INF), 0.0)
        # self.assertEqual(math.pow(0.1, INF), 0.0)
        # self.assertEqual(math.pow(-0.1, INF), 0.0)
        # self.assertEqual(math.pow(-0.9, INF), 0.0)
        # self.assertEqual(math.pow(-1.1, INF), INF)
        # self.assertEqual(math.pow(-1.9, INF), INF)

        # pow(x, y) should work for x negative, y an integer
        self.ftest("(-2.)**3.", math.pow(-2.0, 3.0), -8.0)
        self.ftest("(-2.)**2.", math.pow(-2.0, 2.0), 4.0)
        self.ftest("(-2.)**1.", math.pow(-2.0, 1.0), -2.0)
        self.ftest("(-2.)**0.", math.pow(-2.0, 0.0), 1.0)
        self.ftest("(-2.)**-0.", math.pow(-2.0, -0.0), 1.0)
        self.ftest("(-2.)**-1.", math.pow(-2.0, -1.0), -0.5)
        self.ftest("(-2.)**-2.", math.pow(-2.0, -2.0), 0.25)
        self.ftest("(-2.)**-3.", math.pow(-2.0, -3.0), -0.125)
        # self.assertRaises(ValueError, math.pow, -2.0, -0.5)
        # self.assertRaises(ValueError, math.pow, -2.0, 0.5)

        # the following tests have been commented out since they don't
        # really belong here:  the implementation of ** for floats is
        # independent of the implementation of math.pow
        # self.assertEqual(1**NAN, 1)
        # self.assertEqual(1**INF, 1)
        # self.assertEqual(1**NINF, 1)
        # self.assertEqual(1**0, 1)
        # self.assertEqual(1.**NAN, 1)
        # self.assertEqual(1.**INF, 1)
        # self.assertEqual(1.**NINF, 1)
        # self.assertEqual(1.**0, 1)

    def testSqrt(self):
        # self.assertRaises(TypeError, math.sqrt)
        self.ftest("sqrt(0)", math.sqrt(0), 0)
        self.ftest("sqrt(0)", math.sqrt(0.0), 0.0)
        self.ftest("sqrt(2.5)", math.sqrt(2.5), 1.5811388300841898)
        self.ftest("sqrt(0.25)", math.sqrt(0.25), 0.5)
        self.ftest("sqrt(25.25)", math.sqrt(25.25), 5.024937810560445)
        self.ftest("sqrt(1)", math.sqrt(1), 1)
        self.ftest("sqrt(4)", math.sqrt(4), 2)
        # self.assertEqual(math.sqrt(INF), INF)
        # self.assertRaises(ValueError, math.sqrt, -1)
        # self.assertRaises(ValueError, math.sqrt, NINF)
        # self.assertTrue(math.isnan(math.sqrt(NAN)))

    def testIsfinite(self):
        self.assertTrue(math.isfinite(0.0))
        self.assertTrue(math.isfinite(-0.0))
        self.assertTrue(math.isfinite(1.0))
        self.assertTrue(math.isfinite(-1.0))
        #self.assertFalse(math.isfinite(float("nan")))
        #self.assertFalse(math.isfinite(float("inf")))
        # self.assertFalse(math.isfinite(float("-inf")))

    def testIsnan(self):
        # self.assertTrue(math.isnan(float("nan")))
        # self.assertTrue(math.isnan(float("-nan")))
        # self.assertTrue(math.isnan(float("inf") * 0.0))
        # self.assertFalse(math.isnan(float("inf")))
        self.assertFalse(math.isnan(0.0))
        self.assertFalse(math.isnan(1.0))

    def testIsinf(self):
        # self.assertTrue(math.isinf(float("inf")))
        # self.assertTrue(math.isinf(float("-inf")))
        # self.assertTrue(math.isinf(1e400))
        # self.assertTrue(math.isinf(-1e400))
        #self.assertFalse(math.isinf(float("nan")))
        self.assertFalse(math.isinf(0.0))
        self.assertFalse(math.isinf(1.0))


if __name__ == "__main__":
    unittest.main()
