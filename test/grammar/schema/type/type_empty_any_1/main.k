schema Base:
    name?: str

schema Person:
    info: [] = [{}]
    info: [] = [Base {}, Base {}]
    info: [] = [1, 2, 3]
    info: [] = []
    info: [] = [[]]
    info: [] = [[], {}]
    info: [] = [{"key": "value"}]

person = Person {}
