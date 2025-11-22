import json

jsonStrList = [
    'key: value',
    '- 1\n- 2\n- 3',
    '1',
    '1.1',
    'null',
    'true',
]
_ = [json.dump_to_file(s, "${i}.json") for i, s in jsonStrList]
