schema Base:
    """
    Base schema carrying the shared identifier.

    Attributes
    ----------
    id : int, required
        The unique identifier.
    """
    id: int

schema Address:
    """
    A postal address.

    Attributes
    ----------
    street : str, required
        The street name.
    zipCode : str, required
        The ZIP code.
    geo : {str:float}, optional
        Arbitrary float coordinates keyed by name.
    """
    street: str
    zipCode: str
    geo: {str: float}

schema Person(Base):
    """
    A person in the address book.

    Attributes
    ----------
    name : str, required
        The full name.
    age : int, default is 18
        The age in years.
    emails : [str], optional
        Known email addresses.
    address : Address, required
        The home address.
    """
    name: str
    age?: int = 18
    emails?: [str]
    address: Address

schema UnionS:
    """
    A schema with a union attribute.

    Attributes
    ----------
    val : int | str, required
        An int or a str.
    """
    val: int | str
