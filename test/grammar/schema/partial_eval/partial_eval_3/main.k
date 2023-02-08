schema Spec:
    data: int
    id: int

mixin AMixin:
    spec: Spec {
        id = 1
    }

schema A:
    mixin [AMixin]

    spec: Spec {
        data = 1
    }

a = A()
