# Sample KCL file for testing the Sublime Text syntax grammar.
# Each token below should highlight differently in Sublime Text.

import regex
import yaml

schema Person:
    """A person record.

    This is a docstring attached to the schema. The grammar highlights
    the three-quote delimiters as `punctuation.definition.string.begin`.
    """
    name: str
    age: int = 0
    email: str | None = None

people = [Person {name = "alice", age = 30} for _ in range(3)]

rule check_adult for Person:
    assert person.age >= 0, "age must be non-negative"

raw_path = r"C:\Users\alice\config.k"
multi_line = """
first line
second line
"""

config = {
    apiVersion = "v1"
    kind = "Config"
    metadata = {
        name = "sample"
        labels = {env = "dev"}
    }
}

if any(p.age > 100 for p in people):
    print("found a centenarian")

result = Person | {
    name = "bob"
}
