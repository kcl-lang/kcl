dataList = [1, 2, 3]
dataList0: int = dataList[0]
dataListSlice: [int] = dataList[0:1:2]
dataDict = {key: "value"}
dataKeyValue: str = dataDict["key"]
dataStr: str = "apple"
dataStrSlice = dataStr[::-1] + "apple"[::-1]
keyAny: any = None
dataDictAny = dataDict[keyAny]

# Literal subscript
literalSubscript0 = [1][0]
literalSubscript1 = [1, 2][0]
literalSubscript2 = {key: "value"}.key
literalSubscript3 = {key: "value"}.key1
