schema Parent:
    name: str = "Alice"
    key: str = "key" + " " + name
    name2: str = "Alice"
    key2: str = "key2" + " " + name2
    _tempA = "tempA"
        
schema Son1(Parent):
    name: str = "Son1"
    _tempB = "tempB"

schema Son2(Son1):
    name: str = "Son2" + _tempA
    name2: str = "Son2"

schema Son3(Son2):
    name: str = "Son3" + _tempB

schema Son4(Son3):
    name: str = "Son4" + _tempA + _tempB
    name2: str = "Son4"

parent = Parent {}
son1 = Son1 {}
son2 = Son2 {}
son3 = Son3 {}
son4 = Son4 {}
