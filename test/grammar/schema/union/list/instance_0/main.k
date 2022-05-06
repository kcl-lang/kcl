schema Group:
    nums: [{str:}] = [
        {
            "id": "000",
            "num": 0
        },
        {
            "id": "001",
            "num": 1
        }
    ]

schema AGroup(Group):
    nums: [{str:}] |= [
        {
            "title": "A"
        },
        {
            "title": "B"
        }
    ]

group = AGroup {
    "nums": [
        {
            "description": "A_000"
        },
        {
            "description": "B_000"
        },
    ]
}
