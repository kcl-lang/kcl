# Issue #1979: an empty config default `{}` for a schema-typed attribute must be
# merged with the config override before the child schema is type-checked and its
# check block runs. Previously the empty default was validated on its own and
# failed ("attribute labels is missing" / check on an undefined value).
schema ParentObject:
    child: ChildObject = {}

schema ChildObject:
    labels: {str:str}
    check:
        len(labels) > 0

output = ParentObject {
    child: {
        labels = {"app": "myapp", "env": "prod"}
    }
}
