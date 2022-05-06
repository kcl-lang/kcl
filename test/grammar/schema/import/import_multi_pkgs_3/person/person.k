import ..org.v1 as org_v1
import ..mixins.v1

schema Person:
    mixin [v1.SubjectMixin]

    labels: {str:str}
    mathematics: {str:}
    art: {str:}

    subjects: [org_v1.Subject]