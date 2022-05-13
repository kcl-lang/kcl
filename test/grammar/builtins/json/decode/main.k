import json

jsonStrList = [
    '{"key": "value"}',
    '[1, 2, 3]',
    '1',
    '1.1',
    'null',
    'true',
]
data = [json.decode(s) for s in jsonStrList]
