schema Person:
    labels: {str: str} = {"key": "value"} 
    data: [str] = [name for name in labels if labels[name] == "value"]

person = Person {}
