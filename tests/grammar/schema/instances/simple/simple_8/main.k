# Test case for forward reference of schema instance
# This test verifies that forward references do not create
# duplicate schema instances.

schema Resource [res_name, res_kind]:
    apiVersion: str = "v1"
    kind: str = res_kind
    metadata: {str:str} = {
        name = res_name
    }

# Forward reference: use the instance before it's defined
f = lambda r {
    typeof(r)
}

result_type = f(my_resource)

# Define the instance after the reference
my_resource = Resource("test-resource", "ConfigMap") {}

# Should return 1 instance
instances_count = len(Resource.instances())
