values = {
    res.cpu = 2
    res.no_such_attr = 2
}

schema Config:
    res: Res

schema Res:
    cpu: int

c: Config {
    res: values.res
    res: Res {
        cpu = 1
    }
}
