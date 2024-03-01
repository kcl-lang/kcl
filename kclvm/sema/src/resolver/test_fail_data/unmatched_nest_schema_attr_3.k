schema Name1:
    name1: str

schema Name2:
    name2: str

schema Config:
    n: Name1 | Name2

Config {
    n = {
        n = "n"
    }
}
