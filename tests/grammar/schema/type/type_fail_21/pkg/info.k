schema card:
    num: str

    check:
        num not None

schema info:
    spec: info_spec

schema info_spec:
    info_0: info_0

schema info_0:
    cards: [card]
