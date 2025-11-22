import yaml

dataDict = {"key": "value"}
dataList = [1, 2, 3]

yamlStr = yaml.encode_all([dataDict, dataList])
