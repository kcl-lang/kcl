f = lambda {
    x = 1
    lambda {
        y = 2
        lambda {
            x + y
        }
    }
}
ff = f()()
x = ff()
