name = option("name", type='str', required=True, help="set name value")
a = option("a", default=42, help="set a value")
b = option("b", help="set b value")

# -D c="{'a':2,'b':3}"
c = option("c", default='{"a":1}', type='dict', help="set b value")

# -D obj="{'a':1,'b':1}"
obj = option("obj")

# -D obj="[1,2,3]"
obj2 = option("obj2")
