env1 = "envVar1"
env2 = ""
data = [
    if env1: {
        "key": "ENV_1"
        "value": env1
    }
    {
        "key": "ENV_2"
        "value": env2 or "envVar2"
    }
]
