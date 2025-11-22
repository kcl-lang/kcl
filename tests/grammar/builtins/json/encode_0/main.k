import json

dataDict = {"key": "value"}
dataList = [1, 2, 3]
dataInt = 1
dataFloat = 1.1
dataNone = None
dataBool = True

jsonStr = [json.encode(data) for data in [dataDict, dataList, dataInt, dataFloat, dataNone, dataBool]]
