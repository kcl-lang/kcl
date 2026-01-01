import template

# Test that raw helper preserves quotes
content = template.execute("""{{raw value}}""", {value = "value: \"test\""})
