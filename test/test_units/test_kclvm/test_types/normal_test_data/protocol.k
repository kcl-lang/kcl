schema DataProtocol:
    a: int
    
schema DataMixin for DataProtocol:
    x: int = a

schema Data:
    mixin [DataMixin]
