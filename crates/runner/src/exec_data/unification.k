schema Config:
    data: int

_config: Config {
    data = 1
}
if True:
    _config: Config {
        data = 2
    }

config = _config

schema Map:
    config: Config {
        data = 1
    }

data = Map {}
