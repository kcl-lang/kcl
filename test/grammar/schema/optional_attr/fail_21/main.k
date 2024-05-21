import regex
pattern = 'xxxx'

schema Name:
    name: str

    check:
        regex.match(name, pattern)

n = Name {}
