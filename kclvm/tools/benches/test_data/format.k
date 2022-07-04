schema Data:
    id?: int = 0
    value?: str = "value"

schema Config:
    image: str
    data?: Data

config = Config {image = "image/image:v1"}

