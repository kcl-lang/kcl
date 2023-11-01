x : int = lambda {
    a = 1
    lambda {
        a + 1
    }()
}()
