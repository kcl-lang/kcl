schema WithComponent:
    name: str

schema WithComponents:
    [name: str]: WithComponent = 1
