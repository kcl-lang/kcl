import yaml

yamlStrList = [
    'key1: value2\n---\nkey2: [1, 2, 3]',
    '- 1\n- 2\n- 3\n---\nkey: value',
    '1\n---\n2',
    '1.1\n---\nnull\n---\ntrue\n---\nfalse',
]
data = [yaml.decode_all(s) for s in yamlStrList]
