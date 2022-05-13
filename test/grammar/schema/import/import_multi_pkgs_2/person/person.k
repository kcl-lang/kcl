import ..org.v1 as org_v1
import ..mixins

schema Person:
    mixin [mixins.SubjectMixin]

    labels: {str:str}
    mathematics: {str:}
    art: {str:}

    subjects: [org_v1.Subject]