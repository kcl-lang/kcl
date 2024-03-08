func = lambda config: {str:} {
    x = 1
    lambda {
        y = 1
        lambda {
            z = 1
            [lambda q {
                {value = x + y + z + config.key + q + i}
            }(1) for i in range(1, 2)][0]
        }()
    }()
}

x = func({key = 1})
