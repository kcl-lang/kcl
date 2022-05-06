type Lang = "KCL"|"CUE"

schema CheckArgType[age_:int, name_:Lang]:
    age: any = age_
    name: any =  name_

x1 = CheckArgType(1, "Golang") {}
