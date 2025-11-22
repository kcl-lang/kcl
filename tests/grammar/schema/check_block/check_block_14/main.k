schema Config[data: {str:str}]:
    result: {str:str} = {name = "value"} if data.name == "value" else Undefined

    check:
        result, "Invalid data name value {}".format(data.name)

config = Config({name = "value"})
