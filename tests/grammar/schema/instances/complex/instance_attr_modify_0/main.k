schema Xrd [xrd_kind, group, claims=False]:
    apiVersion: str = "apiextensions.crossplane.io/v1"
    kind: str = "CompositeResourceDefinition"
    metadata: {str:str} = {
       name = "${xrd_kind.lower()}.${group}"
    }
    spec: {str:} = {
        group = group
        names = {
            kind = xrd_kind
        }
    }

_app_xrd = Xrd("XRApp", "apps.group.tech") {
    spec.versions = [
        {
            name = "v1alpha1"
        }
    ]
}

instances = Xrd.instances()

_instance = instances[0]

_instance.kind = "CHANGED"
