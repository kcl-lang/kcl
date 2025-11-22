import file
import json

data_string = json.decode(file.read(file.modpath()+"/test.json"))

key1 = data_string.key1
key2 = data_string.key2
data = data_string.data

workdir = file.workdir()