import template

_data = {
  name = "handlebars", 
  v = [ { a = 1}, { a = 2}], 
  c = { d = 5},
  g = { b = [{ aa = { bb = 55}}, { aa = { bb = 66} } ] },
  people = [ "Yehuda Katz", "Alan Johnson", "Charles Jolley" ]
}

content = template.execute("""\
Hello world from {{name}}

{{#each v}}
{{this.a}}
{{/each}}
{{ c.d }}
{{#each people}}
{{ this }}
{{/each}}
{{#each g.b}}
{{this.aa.bb}}
{{/each}}
""", _data)

content_raw = template.execute("""\
{{this.name}}
{{this.v}}
{{this.c}}
{{this.g}}
{{this.people}}
""", _data) 