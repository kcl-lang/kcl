schema Name:
    name: str = "Kiki"
    age: int = 10
    ID: int = 1

schema Name0:
    name0: Name

schema Name1:
    name1: Name0

schema Name2:
    name2: Name1

schema Name3:
    name3: Name2

schema Name4:
    name4: Name3

schema Name5:
    name5: Name4

schema Name6:
    name6: Name5

schema Name7:
    name7: Name6

schema Person:
    name: Name7

John = Person{
    name.name7.name6:{
        "name5": Name4{
            "name4":{
                "name3": Name2{
                    "name2": {
                        "name1": {
                            "name0":{
                               if True: age = 200
                               elif False: ID = "123"
                               else: "name" : "Tom"
                            }
                        }
                    }
                }
            }
        }
    }
}
