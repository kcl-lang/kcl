import ..org.v1 as org

schema SubjectMixin:
    subjects = [
        org.Subject {
            "name": "mathematics",
            "credit" = 5,
            "major": True,
            "labels": labels,
            "versions": mathematics["versions"]
        },
        org.Subject {
            "name": "art",
            "credit" = art["credit"],
            "major": art["major"],
            "labels": art["labels"],
            "versions": ["1","2"]
        }
    ]