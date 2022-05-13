schema UpperMixin:
    upper = name.upper()

schema Person:
    """Person model
    """

    mixin [UpperMixin]
    """ mixin with UpperMixin
    """

    """ example:
        alice = Person {
            "name": "alice"
        }
    """

    name: str
    """name """

    upper: str
    """upper """

alice = Person {
    "name": "alice"
}
