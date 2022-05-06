schema Config:
    data: str = "aaa"

    check:
        all c in data {c == 'a'}

config = Config {}
