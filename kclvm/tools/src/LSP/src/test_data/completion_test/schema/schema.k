schema Base:
    a: int

schema Person[b: int](Base):
    c: int

p = 

schema Name:
    name: str

schema Config:
    names: [Name]
    names1: {str: Name}
    names2: str | Name

Config{
    names: [{  }]
    names1: {"a": {  }}
} 