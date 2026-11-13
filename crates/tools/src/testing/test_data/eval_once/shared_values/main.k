schema Counter:
    n: int = 0

    check:
        n >= 0, "n is negative"

counter = Counter {}

bump = lambda c: Counter {
    c.n = c.n + 1
    c
}
