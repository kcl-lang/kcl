import math as m
type num = int | float
schema Persons:
    name: str
    
p5: Persons = Persons {
    name: "alice"
}
n: num = 1

func = lambda x{
    x
}

a = func("123")
b = func(x="123")

schema Manifest:
    [name: str] = ""

aaa: any = 1

bbb = any item in [] { True }

import net

c = {
    net: 1
}