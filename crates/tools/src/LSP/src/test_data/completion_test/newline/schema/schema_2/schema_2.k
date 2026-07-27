schema BaseConfig:
    name: str

schema Config(BaseConfig):
    count: int

schema Name:
    config: Config

n = Name{
    config: {
        name = ""
        
    }
}
