schema People:
    name: str

n1: People = {
    name = "kcl"
}

schema Name:
    name: str
    config: Config

schema Config:
    count: int

n2: Name =  { 
    name = "foo"
    config:  {
        count: 1
    }
}
