type num = int | float
schema Persons:
    name: str
    
p5: Persons = Persons {
    name: "alice"
}
n: num = 1