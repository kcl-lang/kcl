x = lambda {
    a = 1
    lambda {
        b = a + 1
        lambda {
            c = b + 1
            lambda {
                d = c + 1
            }()
        }()
    }()
}()
