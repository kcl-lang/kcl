x = lambda {
    a = 1
    lambda {
        b = 2
        lambda a {
            b + a
        }(a)
    }()
}()
