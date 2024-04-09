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
