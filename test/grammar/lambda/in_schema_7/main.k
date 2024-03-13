schema Name:
    y: {str:str} = {}
    z: {str:str} = lambda x {
        y: {str:str} = {}
        y.key = x
        y
    }("value")


n = Name {}
