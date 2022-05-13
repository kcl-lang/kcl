schema Count:
    firstCount: int
    secondCount: int
    check:
        firstCount > 100
        secondCount < 100

count = Count {
    firstCount: 200,
    secondCount: 200
}
