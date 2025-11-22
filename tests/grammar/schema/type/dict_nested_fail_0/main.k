schema Person:
    labels: {str:}
    ids: {str:{str:{str:str}}}

alice = Person {
    "labels": {
        "skin": "yellow",
        "height": 100,
        "female": True,
        "family": {"father":"John"}
    }
    ids: {
        "name1":{
            "name2":{
                "name3":123
            }
        }
    }
}
