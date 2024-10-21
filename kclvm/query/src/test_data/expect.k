schema Data:
    id?: int = 0
    value?: str = "value"

schema Config:
    image: str
    data?: Data

if True:
    configOther = Config {image = "image/other:v1"}

config = Config {
    image = "image/image:v1"
    data = {
        id = 1
        value = "override_value"
    }
}

config: Config {
    image = "image/image:v3"
}
dict_config = {
    "image": "image/image:v2"
    "data": {
        "id": 2
        "value2": "override_value2"
    }
}
envs = [
    {
        key = "key1"
        value = "value1"
    }
    {
        key = "key2"
        value = "value2"
    }
]

isfilter = False

count = 2

msg = "Hi World"

dict_delete = {
    "data": {
        "id": 1
        "value": "override_value"
    }
}

insert_config = {
    key = 1
}

uni_config = {
    labels: {key1: 1}
}

config_unification: Config {
    "image": "image/image:v4"
    env: {
        "aaa": "aaa"
        bbb: "bbb"
    }
}
