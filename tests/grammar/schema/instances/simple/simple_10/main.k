# Test case for lambda with forward reference (original user scenario)
# This test verifies the exact scenario reported by the user:
# When a schema instance is referenced in a lambda before it's defined,
# it should only create one instance, not multiple duplicates.

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

# Lambda that references the schema instance before it's defined
f = lambda xrd {
  "result: ${typeof(xrd)}"
}

# Call lambda with forward reference
s = f(_app_xrd)

# Define the instance after the lambda call
_app_xrd = Xrd("XRApp", "apps.chainstar.tech") {
    spec.versions = [
        {
            name = "v1alpha1"
        }
    ]
}

# Should return exactly 1 instance
instances = len(Xrd.instances())
