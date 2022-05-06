import base_schema_pkg.base

schema Student(base.Person):
    """Student is the person with a grade

    Attributes
    ----------
    grade : int, default is Undefined, required
        the current grade that the student is in.

    """
    grade: int
