schema Config:
    data: str = "aaa"

    check:
        any c in data {c == 'a'}

config = Config {}
