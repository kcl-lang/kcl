func = lambda config: {str:} {
    x = 1
    lambda {
        y = 1
        lambda {
            z = 1
            lambda {
                {value = x + y + z + config.key}
            }()
        }()
    }()
}

x = func({key = 1})
