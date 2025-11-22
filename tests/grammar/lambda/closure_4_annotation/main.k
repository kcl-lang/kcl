x: int = lambda {
    a = 1
    lambda {
        b = a + 1
        lambda {
            c = b + 1
        }()
    }()
}()
