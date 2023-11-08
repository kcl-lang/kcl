import json

# Right cases

resultRight1: bool = json.validate("1")
resultRight2: bool = json.validate("true")
resultRight3: bool = json.validate("1.20")
resultRight4: bool = json.validate("null")
resultRight5: bool = json.validate("[0, 1, 2]")
resultRight6: bool = json.validate('{"key": "value"}')

# Wrong cases

resultWrong1: bool = json.validate("1@")
resultWrong2: bool = json.validate("True")
resultWrong3: bool = json.validate("1.20.23+1")
resultWrong4: bool = json.validate("None")
resultWrong5: bool = json.validate("[0, 1, 2,]")
resultWrong6: bool = json.validate(r'''{"key": 'value'}''')
