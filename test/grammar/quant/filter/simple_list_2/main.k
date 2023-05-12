schema Student:
    name: str
    id: int
    grade: int

    check:
        id >= 0
        0 <= grade <= 100

students: [Student] = [
    {name = "Alice", id = 1, grade = 85}
    {name = "Bob", id = 2, grade = 70}
    {name = "Charlie", id = 3, grade = 90}
    {name = "David", id = 4, grade = 80}
    {name = "Eve", id = 5, grade = 95}
]

query_student_where_name = lambda students: [Student], name: str -> Student {
    (filter s in students {
        s.name == name
    })?[0]
}

alice: Student = query_student_where_name(students, name="Alice")
bob: Student = query_student_where_name(students, name="Bob")
