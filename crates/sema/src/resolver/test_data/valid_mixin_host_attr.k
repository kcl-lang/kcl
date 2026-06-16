# Valid: mixin accesses attributes from the host schema at runtime.
# This must NOT produce a type error.
mixin FullnameMixin:
    fullName = "{} {}".format(firstName, lastName)

schema Person:
    mixin [FullnameMixin]
    firstName: str
    lastName: str
