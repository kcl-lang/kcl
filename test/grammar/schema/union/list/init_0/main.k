schema Group:
    mixin [AGroupMixin]
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

schema AGroupMixin:
    nums |= [
        {
            "title": "A"
        },
        {
            "title": "B"
        }
    ]

group = Group {
    "nums": [
        {},
        {
            "description": "A_000"
        }
    ]
}
