import ..org.v1 as org
import ..mixins

schema Person:
    mixin [mixins.SubjectMixin]

    labels: {str:str}
    mathematics: {str:}
    art: {str:}

    subjects: [org.Subject]