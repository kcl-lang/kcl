# Test for lambda forward reference in nested schemas
# This test ensures that forward references work correctly
# in nested schema structures.

schema Inner:
    innerValue: str = ""
    value = Format(innerValue)

schema Outer:
    inner: Inner = {}
    outerValue: str = ""
    formatted = Format(outerValue)

# Instances created before Format is defined
instance1 = Outer{
    inner: Inner{innerValue = "inner1"},
    outerValue = "outer1"
}

# Format is defined after the schema instances
Format = lambda s: str {
    "formatted: ${s}"
}

# Instance created after Format is defined
instance2 = Outer{
    inner: Inner{innerValue = "inner2"},
    outerValue = "outer2"
}
