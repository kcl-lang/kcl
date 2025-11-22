import yaml

yamlStrList = [
    'key: value',
    '- 1\n- 2\n- 3',
    '1',
    '1.1',
    'null',
    'true',
]
_ = [yaml.dump_to_file(s, filename="${i}.yaml") for i, s in yamlStrList]
