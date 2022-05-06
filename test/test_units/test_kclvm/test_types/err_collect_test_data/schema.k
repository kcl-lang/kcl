import pkg

schema DataProtocol:
    data: str

schema DataMixin:
    name: str = "data"

# Invalid schmea inherit from protocol type error
schema Data for DataProtocol:
    mixin [DataMixin]
    # Type check errors
    a: int = "s"
    b: str = 1
    c: bool = "s"
    d: [int] = {}
    e: {:} = []
    f: "Yellow" | "Green" = "Red"
    g: int = 1 if True else "s"
    i: int | str = 1.0


# Invalid protocol type error
schema SomeMixin1 for pkg.pkg.SomeProtocol:
    some: int

# Pacakge attribute not found type error
schema SomeMixin2 for pkg.NotFoundProtocol:
    some: int

# Unique key error
schema _Data:
    name: str

schema _Data:
    id: int
