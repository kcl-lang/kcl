schema Person:
    info: {str:str} = {
        "name": "default name"
    }
    hc: [int] = [1, 2, 3]
    hc1: [int] = [1, 2, 3]
    hc2: [int] = [1, 2, 3]
    hc3: [int] = [1, 2, 3]
    hc4: [int] = [1, 2, 3]
    hc5: [int] = [1, 2, 3]

person = Person {
    hc += [4]
    hc += [5]
    hc1 += [4]
    hc2 += [4]
    hc3 = [4, 5, 6]
}
