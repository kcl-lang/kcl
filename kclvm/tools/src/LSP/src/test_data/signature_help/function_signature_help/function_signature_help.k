func = lambda x, y, z{
      x * y + z
}

a = func
b = func(a)
c = func(a, 1)
c = func(a, 1, z = 1)

