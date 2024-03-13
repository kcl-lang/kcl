ll0 = [lambda {
    i + 1
}() for i in range(2)]
ll1 = [lambda {
    lambda {
        i + 1
    }()
}() for i in range(2)]
ll3 = [lambda {
    lambda {
        lambda {
            i + 1
        }()
    }()
}() for i in range(2)]
