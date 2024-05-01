
func1 = lambda x {
    x
}

func2 = lambda e, func: (any) -> any {
    func(e)
}

x = func2(1, func1)
