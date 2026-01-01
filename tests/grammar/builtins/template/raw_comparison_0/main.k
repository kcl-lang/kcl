import template

# Test comparison: normal {{var}} vs {{raw var}}
escaped = template.execute("""{{value}}""", {value = "<div>test</div>"})
unescaped = template.execute("""{{raw value}}""", {value = "<div>test</div>"})
unescaped_triple = template.execute("""{{{value}}}""", {value = "<div>test</div>"})
