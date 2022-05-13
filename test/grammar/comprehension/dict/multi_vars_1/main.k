data = {"key1": "value1", "key2": "value2"}
dataKeys = [k for k, v in data]  # ["key1", "key2"]
dataValues = [v for k, v in data]  # ["value1", "value2"]
dataFilter = {k: v for k, v in data if k == "key1" and v == "value1"}  # {"key1": "value1"}
