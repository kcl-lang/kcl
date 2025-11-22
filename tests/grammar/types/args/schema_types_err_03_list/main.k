schema CheckArgType[a0: [], a1: [any], a2: [str]]:
    attr0: any = a0
    attr1: any = a1
    attr2: any = a2

x1 = CheckArgType([123, True], ["abc", 456], ["aa", True]) {}
