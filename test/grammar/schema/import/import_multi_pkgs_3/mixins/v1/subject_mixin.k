import ...org.v1

schema SubjectMixin:
    subjects = [
        v1.Subject {
            "name": "mathematics",
            "credit" = 5,
            "major": True,
            "labels": labels,
            "versions": mathematics["versions"]
        },
        v1.Subject {
            "name": "art",
            "credit" = art["credit"],
            "major": art["major"],
            "labels": art["labels"],
            "versions": ["1","2"]
        }
    ]