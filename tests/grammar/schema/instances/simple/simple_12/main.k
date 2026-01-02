# Test case for schema with mixins
# This test verifies that schema mixins do not create
# duplicate instances.

schema Metadata:
    labels: {str:str} = {}
    annotations: {str:str} = {}

schema Resource [res_name]:
    name: str = res_name
    Metadata

# Create instances
r1 = Resource("resource-1") {}

r2 = Resource("resource-2") {}

# Should have 2 instances for each schema
resource_instances = len(Resource.instances())
mixin_instances = len(Metadata.instances())
