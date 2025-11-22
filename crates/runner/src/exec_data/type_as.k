schema Data[idVar: int]:
    id?: int = idVar

type IntList = [int]
type StrMap = {str:str}

a = 1 as int
b = 2.0 as float
c = True as bool
d = "s" as str
e = [1] as IntList
f = {key: "value"} as StrMap
g = Data(1) as Data
