import yaml

yamlStrList = [
    'key: value',
    '- 1\n- 2\n- 3',
    '1',
    '1.1',
    'null',
    'true',
]
data = [yaml.decode_all(s) for s in yamlStrList]
