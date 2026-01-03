# Test for lambda forward reference in schema
# This test ensures that when a schema references a lambda function
# that is defined later in the code, the schema field should still
# be evaluated correctly using lazy evaluation.

schema Name:
    m = MyFunc("name_value")

# Instance created before MyFunc is defined
n = Name{}

# MyFunc is defined after the schema instance
MyFunc = lambda f: str {
    "constant"
}

# Test calling MyFunc directly
m = MyFunc("input_value")
