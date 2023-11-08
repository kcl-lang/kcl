import yaml

# Right cases

resultRight1: bool = yaml.validate("1")
resultRight2: bool = yaml.validate("true")
resultRight3: bool = yaml.validate("1.20")
resultRight4: bool = yaml.validate("null")
resultRight5: bool = yaml.validate("[0, 1, 2]")
resultRight6: bool = yaml.validate('{"key": "value"}')
resultRight7: bool = yaml.validate('a:1\n---\nb:2')

# Wrong cases

resultWrong1: bool = yaml.validate("a:\n1")
resultWrong2: bool = yaml.validate("a:\n1\n  - 2")
resultWrong3: bool = yaml.validate("a:\n-1")
resultWrong4: bool = yaml.validate("1a   : \n1")
resultWrong5: bool = yaml.validate("a:\n- 1\n-----\na:\n- 1")
resultWrong6: bool = yaml.validate(r'''{"key" + 'value'}''')
resultWrong7: bool = yaml.validate("a:1\n-----\nb:\n-2")
