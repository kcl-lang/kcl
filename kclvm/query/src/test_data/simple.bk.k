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
    data = {id = 1, value = "override_value"}
    data = {id = 1, value = "override_value"}
}

config : Config {
    image = "image/image:v3"
}

dict_config = {
    "image": "image/image:v1"
    "data": {
        "id": 1
        "value": "override_value"
    }
}
envs = [
    {
        "name": "ENV1"
        "value": "value1"
    }
    {
        "name": "ENV2"
        "value": "value2"
    }
]

isfilter = True

count = 1

msg = "Hello World"

delete = "Delete"

dict_delete = {
    "image": "image/image:v1"
    "data": {
        "id": 1
        "value": "override_value"
    }
}

dict_delete_whole = {
    "image": "image/image:v1"
    "data": {
        "id": 1
        "value": "override_value"
    }
}

insert_config = {} 

uni_config = {
    labels: {key1: "value1"}
}

config_unification: Config {
    image = "image/image:v1"
    data = {id = 1, value = "override_value"}
}

config_unification_delete: Config {
    image = "image/image:v1"
    data = {id = 1, value = "override_value"}
}