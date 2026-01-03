# Test for lambda closure with forward reference in schema
# This test ensures that lambda functions defined later can properly
# capture variables and work correctly when used in schemas.

schema Processor:
    input: int = 0
    # Use lambda that captures outer variable
    result = Transform(input)

# Transform is defined after the schema definition
multiplier = 5
Transform = lambda x: int {
    x * multiplier
}

# Instance created before Transform is defined
p1 = Processor{input = 10}

# Another instance after Transform is defined
p2 = Processor{input = 20}
