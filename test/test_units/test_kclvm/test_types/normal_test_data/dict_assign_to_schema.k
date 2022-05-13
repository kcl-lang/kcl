# Dict to schema test
schema Person:
    name: str
    age: int

data0: {str:str} = {name: "Alice"}
data1: {str:int} = {age: 18}
data2: {:} = {}
data3: {any:any} = {}
data4: any = {}

person0 = Person {**data0}
person1 = Person {**data1}
person2 = Person {**data2}
person3 = Person {**data3}
person4 = Person {**data4}

schema KeyVal:
    labels?: {str:}

schema Data:
    keyVals?: KeyVal

# Dict to schema in schema test
data = Data {
    keyVals: {
        labels.key: 456
    }
    keyVals.labels.key2: 789
}

# Any dict to schema
personData: any = {name: "Alice", age: 18}
person: Person = personData
