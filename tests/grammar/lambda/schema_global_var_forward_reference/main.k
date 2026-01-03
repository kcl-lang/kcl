# Test for global variable forward reference in schema
# This test ensures that when a schema references a global variable
# that is defined later in the code, the schema field should still
# be evaluated correctly using lazy evaluation.

schema Config:
    value = globalValue

# Instance created before globalValue is defined
c = Config{}

# globalValue is defined after the schema instance
globalValue = "defined"

# Another instance after globalValue is defined
c2 = Config{}
