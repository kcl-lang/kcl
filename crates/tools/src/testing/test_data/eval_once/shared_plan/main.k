# Planning unifies this config expression with the global `a`.
{a = [1]}
a = [1]

change = lambda xs {
    xs[0] = 2
}
