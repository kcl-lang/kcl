schema Name:
    name: str

schema Config:
    n: {str:Name}

Config {
    n = {
        n.n = "n"
    }
}
