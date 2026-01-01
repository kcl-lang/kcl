import template

# Verify that normal template still escapes HTML (backward compatibility)
content = template.execute("""{{value}}""", {value = "<div>test</div>"})
