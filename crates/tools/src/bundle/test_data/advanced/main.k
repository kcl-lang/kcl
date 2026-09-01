import math

import .base

schema Triangle(base.Shape):
    sides = 3

n: base.Count = 3
area_factor = base.double(n)
circumference = Triangle {}.sides * n + math.floor(0.5)
