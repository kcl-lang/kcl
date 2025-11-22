import sub as pkg

_a = 1

t1 = typeof(_a)
t2 = typeof("abc")

schema Person:
    name?: any

_x1 = Person{}
t3 = typeof(_x1)

_x2 = pkg.Person{}
t4 = typeof(_x2)
t5 = typeof(_x2, full_name=True)

none1 = typeof(None)

bool1 = typeof(True)
bool2 = typeof(False)
bool3 = typeof(False, full_name=True)

int1 = typeof(0)
int2 = typeof(0, full_name=True)

float1 = typeof(0.0)
float2 = typeof(0.0, full_name=True)

str1 = typeof("")
str2 = typeof("abc", full_name=True)

list1 = typeof([])
list2 = typeof([123, "abc"])
list3 = typeof([123, {}], full_name=True)

dict1 = typeof({})
dict2 = typeof({"a":1}, full_name=True)
