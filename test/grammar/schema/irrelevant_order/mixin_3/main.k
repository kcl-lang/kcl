schema ConfigMixin:
    output: str = option("output") or "default_output"
    data = {"output": output}

schema Config:
    mixin [ConfigMixin]
    name: str = "config"
    data: {str:} = {}

config = Config {}
