import regex

schema Data:
    [attr: str]: str
    check:
        regex.match(attr, r'^[-._a-zA-Z0-9]+$')

data = Data {
    "key1": "value1"
    "key2": "value2"
    "invalid/label": "value3"
}
