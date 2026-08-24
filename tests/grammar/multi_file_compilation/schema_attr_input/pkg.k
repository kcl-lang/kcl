# Compute the workload attributes using the schemas and base config from
# main.k. The parameterized ServerBackend takes its Server input via the
# `inputConfig` argument (a schema-attr input); the inner `app` attribute
# is provided through the schema-instance block.
result = ServerBackend(inputConfig=appConfiguration) {
    app = "app"
}.workloadAttributes