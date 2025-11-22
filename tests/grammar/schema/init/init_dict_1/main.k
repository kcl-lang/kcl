schema Person:
    name?: str = None
    info?: {str:str} = {"name": name} if name else None

JohnDoe = Person {}
