schema Data:
    id?: int = 0
    value?: str = "value"

schema Config:
    image: str
    data?: Data

if True:
    configOther = Config {
        image = "image/other:v1"
    }

config = Config {
    image = "image/image:v1"
    data = {
        id = 1
        value = "override_value"
    }
}
