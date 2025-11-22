lists = ["Alice", "Bob", "Green"]

schema Person:  
    name: str = lists[0].upper()[1]

person = Person {}
