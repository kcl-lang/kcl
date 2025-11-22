import yaml

yamlStr = """\
key: value
"""
data = yaml.decode(yamlStr)
dataList = yaml.decode_all(yamlStr)
