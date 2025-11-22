schema Person:
    mixin [Fullname]
    firstName: str
    lastName: str

schema Fullname: # It is a mixin, but 'Fullname' is not end with 'Mixin
    fullName = "{} {}".format(firstName, lastName)
