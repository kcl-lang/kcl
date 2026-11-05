schema Counter:
    n: int = 0

counter = Counter {}

bump = lambda c: Counter {
    c.n = c.n + 1
    c
}
