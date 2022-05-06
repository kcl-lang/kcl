schema DataMap:
    [name: str]: str
    
    check:
        name.islower()

data = DataMap {
    "key1": "1"
    "key2": "2"
}
