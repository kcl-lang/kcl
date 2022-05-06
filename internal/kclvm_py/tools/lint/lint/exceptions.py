class InvalidCheckerError(Exception):
    """raised when selected reporter is invalid (e.g. not found)"""

    def __init__(self, checker: str):
        self.checker_name = checker

    def __str__(self):
        return f"Args wrong, checker {self.checker_name} not found"


class EmptyReporterError(Exception):
    """raised when reporter list is empty and so should not be displayed"""

    def __str__(self):
        return "Without output reporter"


class InvalidReporterError(Exception):
    """raised when selected reporter is invalid (e.g. not found)"""

    def __init__(self, reporter: str):
        self.reporter_name = reporter

    def __str__(self):
        return f"Args wrong, reporter {self.reporter_name} not found"
