import ..org.v1 as org
import ..metadata.v1 as meta
import ..mixins

schema Person:
    mixin [mixins.SubjectMixin]
    name?: str = None
    tagged?: bool = False
    labels?: {str:str} = None
    famous?: bool = False
    address?: str = None
    partitions?: [str] = None
    principals?: {str:str} = None

    mathematics?: {str:} = None
    art?: {str:} = None
    subjects?: [org.Subject] = None

    metadata?: meta.ObjectMeta = {
        "name": name,
        "tagged": tagged,
        "labels": labels
    }
    scholarModel?: org.Scholar = {
        "school": {
            "metadata": {
                "labels": labels
            },
            "address": address,
            "famous": famous,
            "subjects": subjects,
            "partitions": partitions,
            "principals": principals
        },
        "metadata": metadata
    }
