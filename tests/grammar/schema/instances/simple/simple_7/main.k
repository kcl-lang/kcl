# Test case for schema with multiple assignment statements
# This test verifies that schema with multiple attributes
# only creates one instance, not multiple duplicates.

schema Xrd [xrd_kind, group, claims=False]:
    apiVersion: str = "apiextensions.crossplane.io/v1"
    kind: str = "CompositeResourceDefinition"
    metadata: {str:str} = {
       name="${xrd_kind.lower()}.${group}"
    }
    spec: {str:} = {
        group = group
        names = {
            kind = xrd_kind
        }
    }

# Create a single instance
xrd_instance = Xrd("XRApp", "apps.example.com") {
    spec.versions = [
        {
            name = "v1alpha1"
        }
    ]
}

# Should return 1 instance, not multiple
instances_count = len(Xrd.instances())
