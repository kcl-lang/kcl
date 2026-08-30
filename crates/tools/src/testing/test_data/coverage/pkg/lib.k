# Library code under test. Every line below is a top-level statement and
# therefore a candidate for coverage tracking. Test cases live in `lib_test.k`.
a = 1
b = 2
c = 3
d = 4
e = 5
sum = lambda x, y {
    x + y
}
negate = lambda x {
    0 - x
}
# A helper with three execution paths. The lambda body has to end in
# an expression, so we wrap an `if` statement that is only relevant
# when the caller invokes `grade`. Tests only call `grade("a")`,
# leaving the "b"/"c" branches uncovered.
grade = lambda label -> str {
    bucket = 0
    if label == "a":
        bucket = 1
    if label == "b":
        bucket = 2
    if label == "c":
        bucket = 3
    "ok:{}".format(bucket)
}
