# Test for lambda functions in schema methods to ensure correct parameter passing
# when called multiple times on different instances

schema Builder:
    """A builder schema with a lambda method that adds items"""
    items: [str] = []

    add: (str) -> Builder = lambda item: str {
        """Add an item to the builder and return a new builder"""
        new_items = items + [item]
        Builder{items = new_items}
    }

# Test: Chain multiple calls
b1 = Builder{}
b2 = b1.add("first")
b3 = b2.add("second")
b4 = b3.add("third")
