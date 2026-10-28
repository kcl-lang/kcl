import units
# Sub-unit suffixes must not lose a unit to floating point division
n1 = units.to_n(1)
n2 = units.to_n(2)
n3 = units.to_n(0.5)
u1 = units.to_u(1e-3)
u2 = units.to_u(1)
m1 = units.to_m(0.7)
m2 = units.to_m(1)
m3 = units.to_m(0.3)
