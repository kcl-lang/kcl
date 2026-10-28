import .pkg

f = lambda v {
    _t = v
    pkg.mutate(_t)
    v
}
a = [1]
b = f(a)
c = a
