import ..org.v1 as org_v1

schema SubjectMixin:
    subjects = [
        org_v1.Subject {
            "name": "mathematics",
            "credit" = 5,
            "major": True,
            "labels": labels,
            "versions": mathematics["versions"]
        },
        org_v1.Subject {
            "name": "art",
            "credit" = art["credit"],
            "major": art["major"],
            "labels": art["labels"],
            "versions": ["1","2"]
        }
    ]