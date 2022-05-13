schema ProviderConfig:
    __settings__?: {str:str} = {"output_type": "INLINE"}

schema AConfig(ProviderConfig):
schema BConfig(ProviderConfig):
    access_key?: str = option("ak")
    secret_key?: str = option("sk")