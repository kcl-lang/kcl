x: int = lambda {
    a = 1
    lambda {
        b = 2
        c = 3
        lambda a {
            a + b + c
        }(a)
    }()
}()
