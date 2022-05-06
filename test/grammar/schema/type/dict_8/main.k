schema Role1:
    name: str
    
schema Role2:
    name: str

schema Config:
    data: Role1 | Role2

data = Config {
    data: Role1 {
        name: "name"
    }
}
