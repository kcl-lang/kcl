import yaml
import file

data_string = yaml.decode(file.read("test.yaml"))

key1 = data_string.key1
key2 = data_string.key2
data = data_string.data