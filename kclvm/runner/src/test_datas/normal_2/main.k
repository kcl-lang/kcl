
schema NumberMap:
    [num: str]: int

    check:
        int(num) % 2 == 0

numMap = NumberMap {
    str(0): 0
    str(2): 2
}
