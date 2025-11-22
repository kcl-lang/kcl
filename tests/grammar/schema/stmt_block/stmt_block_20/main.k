schema Person:
    labels: {str:str} = {"key": "value"} 
    data: {str:str} = {name: labels[name] for name in labels if labels[name] == "value" and name}

person = Person {}
