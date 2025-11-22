import template

content = template.execute("hello {{world}}", {world = "世界"})
