schema Group:
    nums: [str] = ["value1"]

schema AGroup(Group):
    nums: [str] |= ["value1", "value2"]

group = AGroup {}
