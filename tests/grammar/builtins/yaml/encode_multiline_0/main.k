import yaml

# A KCL string containing the literal two-character sequence `\n`
# (backslash + n). Without `multiline_string=True` the YAML emitter
# leaves these escapes in the output:
plain = yaml.encode({"field": "line1\\nline2\\nline3"})

# Passing `multiline_string=True` (as the 5th positional argument)
# asks the emitter to convert the `\n` escape into a real newline
# and use block-scalar style:
block = yaml.encode({"field": "line1\\nline2\\nline3"}, False, False, False, True)
