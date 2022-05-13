import pkg

rule SomeRule1 for pkg.SomeProtocol:
    some > 0

# Invalid protocol type error
rule SomeRule2 for pkg.pkg.SomeProtocol:
    some > 0

# Package attribute not found type error
rule SomeRule3 for pkg.NotFoundProtocol:
    some > 0

# Invalid rule protocol object type
rule SomeRule4 for pkg.variable:
    some > 0
