# Issue 1078
# Errors caused by deep copy of circular references within schema and schema lambda
schema A:
    a: int = 1
    f = lambda {
        a
    }

schema B[a: A]:

C = lambda a: A {
}
a = A {}

C(a)
B(a)
