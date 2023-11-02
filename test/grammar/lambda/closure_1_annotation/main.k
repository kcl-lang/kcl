x: int = lambda {
    a = 1
    b = 2
    lambda x {
        a + b + x
    }(3)
}()
