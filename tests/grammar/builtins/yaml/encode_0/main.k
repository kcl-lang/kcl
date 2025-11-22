import yaml

dataDict = {"key": "value"}
dataList = [1, 2, 3]

yamlStr = [yaml.encode(data) for data in [dataDict, dataList]]
