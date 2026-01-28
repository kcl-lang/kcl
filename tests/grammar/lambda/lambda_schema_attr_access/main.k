# Test lambda accessing schema attributes
# This tests the fix for the bug where lambda parameters were incorrectly
# captured from closure instead of using the current value

schema Processor:
    name: str = "default"
    data: [str] = []

    process: (str) -> Processor = lambda item: str {
        # Lambda should access the current schema's name and data
        new_data = data + [item]
        Processor{name: name, data: new_data}
    }

    processWithPrefix: (str) -> Processor = lambda item: str {
        # Lambda should access the current schema's name
        prefixed = "{name}:{item}"
        new_data = data + [prefixed]
        Processor{name: name, data: new_data}
    }

# Test 1: Default processor
p1 = Processor{}
p2 = p1.process("item1")
p3 = p2.process("item2")

# Test 2: Named processor
n1 = Processor{name: "my_processor"}
n2 = n1.process("first")
n3 = n2.process("second")

# Test 3: Process with prefix
x1 = Processor{name: "test", data: []}
x2 = x1.processWithPrefix("a")
x3 = x2.processWithPrefix("b")
