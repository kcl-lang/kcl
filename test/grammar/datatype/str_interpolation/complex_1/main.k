CONSTANT = {"key1": "value1", "key2": "value2"}

schema Data:
    jsonStr: str = "${CONSTANT: #json}"
    yamlStr: str = "${CONSTANT: #yaml}"

data = Data {}
