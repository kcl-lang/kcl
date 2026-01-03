# Test case for schema inheritance
# This test verifies that schema inheritance does not
# create duplicate instances.

schema BaseResource [name]:
    metadata: {str:str} = {
        name = name
    }
    apiVersion: str = "v1"

schema DerivedResource [name, res_kind]:
    BaseResource(name)
    kind: str = res_kind
    spec: {str:} = {}

# Create instances of derived schema
instance1 = DerivedResource("resource-1", "Deployment") {
    spec.replicas = 3
}

instance2 = DerivedResource("resource-2", "Service") {
    spec.type = "ClusterIP"
}

# BaseResource should have 0 instances (only derived instances are created)
# DerivedResource should have 2 instances
base_instances = len(BaseResource.instances())
derived_instances = len(DerivedResource.instances())
