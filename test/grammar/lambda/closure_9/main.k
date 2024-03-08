func = lambda config: {str:} {
    x = 1
    lambda {
        y = 1
        lambda {
            z = 1
            lambda q {
                {value = x + y + z + config.key + q}
            }(1)
        }()
    }()
}

x = func({key = 1})
