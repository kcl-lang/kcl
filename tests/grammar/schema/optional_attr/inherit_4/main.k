schema Name:
    n1: str = "aa"
    n2: str

schema Sub[a: str](Name):
    n2 = a

schema Phase:
    a: Sub

a = lambda {
    [[Phase{a = Sub("a")}.a]][0][0] | {}
}()
