# Test for multiple lambda forward references in schema
# This test ensures that when a schema references multiple lambda functions
# that are defined later in the code, all schema fields should be evaluated correctly.

schema Calculator:
    a = FuncA(10)
    b = FuncB(20)
    c = FuncC(30)

# Instance created before functions are defined
calc = Calculator{}

# Functions are defined after the schema instance
FuncA = lambda x: int {
    x + 1
}

FuncB = lambda x: int {
    x * 2
}

FuncC = lambda x: int {
    x - 5
}

# Test calling functions directly
resultA = FuncA(100)
resultB = FuncB(100)
resultC = FuncC(100)
