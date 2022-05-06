schema Data:
    key: str = "value"

result1 = "key" in {key: "value"}
result2 = "key" not in Data {}
result3 = "key" is None
result4 = "key" is not Undefined
