schema Person:
    name?: str

data: {str:str} | Person = {name: "Alice"}
a = {
    if True: **None
    elif False: **data
    else: **{}
}
b = {if True: **1}
