str0 = [str(i) + ": " + c.upper() for i, c in "apple"]
str1 = [c.upper() for c in "apple"]
str2 = {str(i): c.upper() for i, c in "apple"}
str3 = {c: c.upper() for c in "apple"}

list0 = [str(i) + ": " + c.upper() for i, c in ["a", "p", "p"]]
list1 = [c.upper() for c in ["a", "p", "p"]]
list2 = {str(i): c.upper() for i, c in ["a", "p", "p"]}
list3 = {c: c.upper() for c in ["a", "p", "p"]}

dict0 = [str(k) + ": " + v.upper() for k, v in {"a": "A", "p": "P"}]
dict1 = [c.upper() for c in {"a": "A", "p": "P"}]
dict2 = {str(k): v.upper() for k, v in {"a": "A", "p": "P"}}
dict3 = {c: c.upper() for c in {"a": "A", "p": "P"}}

listUnion0 = [i for i in [0] or ["1"]]
listUnion1 = [i for i in [0, "1"] or ["1", 0]]
