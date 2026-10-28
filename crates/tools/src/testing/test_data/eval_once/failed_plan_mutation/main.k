# Planning unifies this config expression with the global `a`, which
# conflicts, so every case whose own code passes fails to plan.
{a = 1}
a = 2

schema Counter:
    n: int = 0

counter = Counter {}

bump = lambda c: Counter {
    c.n = c.n + 1
    c
}
