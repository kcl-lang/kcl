schema Person[nameVar: str]:
    name: str = nameVar
    age: int = 18
    
_person = Person("Alice")
_person.name = "Bob"

data = {
    key1: "value1"
    key2: "value2"
}
schemaSelectAttr: int = _person.age
dictSelectAttr: str = data.key1
strSelectAttr0: any = "abc".upper()
strSelectAttr1: any = strSelectAttr0.upper()

unionData: {str:} | Person = Person("Bob")
unionDataAttr: int = unionData.age
