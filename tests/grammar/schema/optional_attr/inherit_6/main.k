schema Name:
    n1: str = "aa"
    n2: str

schema Sub[a: str](Name):
    n2 = a

schema Phase:
    a: Sub

schema Schema:
    f0: () = lambda {
        [[Phase{a = Sub("a")}.a]][0][0]
    }
    f1: () = lambda {
        [[Phase{a = Sub("a")}.a]][0][0] | {}
    }
    f2: () = lambda {
        s = [[Phase{a = Sub("a")}.a]][0][0] 
        s | {}
    }
    ff: () -> () = lambda {
        lambda {
            [[Phase{a = Sub("a")}.a]][0] | [{}]
        }
    }

a = Schema().f0()
b = Schema().f1()
c = Schema().f2()
d = Schema().ff()()
