
schema CheckArgType[age_:int, name_:"KCL"|"CUE"]:
    age: any = age_
    name: any =  name_

x1 = CheckArgType(1, "KCL") {}
