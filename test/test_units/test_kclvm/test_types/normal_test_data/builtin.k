import math
import net


schema Person:
    name: str
    age: int

a: any = math.log(10)
b: any = 1 + 2
c: any = net.is_IP("192.168.0.1")
d: str = ".".replace(".", "")
e: bool = "A".isupper()
f: [] = Person.instances()
g: any = 1.1
h: int = int(1.1)
i: float = float(1.1)
