import pkg

schema DataMixin:
    name: str

schema Data:
    mixin [
        pkg.PersonMixin,
        DataMixinError
    ]
