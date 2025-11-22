items = [1, 2, 3]
func = lambda config: {str:} {
    [lambda {
        config.key + i
    }() for i in items]
}

x = func({key = 1})
