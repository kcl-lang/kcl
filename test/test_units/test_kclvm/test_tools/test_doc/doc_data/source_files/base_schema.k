schema Person:
    name: str
    age: int

schema Student(Person):
    """Student is the person with a grade

    Attributes
    ----------
    grade : int, default is Undefined, required
        the current grade that the student is in.

    """
    grade: int
