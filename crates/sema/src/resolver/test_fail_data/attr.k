schema Data:
    id: int = 1

dataList = [{data = 1} for data in [Data {}]]
dataMap = {data = 1 for data in [Data {}]}
