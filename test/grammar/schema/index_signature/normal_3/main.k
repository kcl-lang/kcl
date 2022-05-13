
schema NumberMap:
    [num: str]: int | str

numMap = NumberMap {
    **{str(i): str(i * 2) if i < 2 else i * 2 for i in range(0, 5)}
}
