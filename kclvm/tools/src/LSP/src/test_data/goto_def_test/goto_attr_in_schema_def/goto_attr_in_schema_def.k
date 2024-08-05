schema Name:
    name1: str
    name2: str
    name3: str

name1 = "1"

name = Name{
    name1 = name1 # (5,0) (5,5)
    name2 = name1 # (8,4) (8,9)
    name3 = name2 # (9,4) (9,9)
}