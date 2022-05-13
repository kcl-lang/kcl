import yaml

data = {
    "key1": "value1"
    "key2": "value2"
    "data": [1, 2, 3]
}
data_string = yaml.encode(data)
