import manifests

docs = [{a = 1}, {b = 2}]
# This output does not plan.
manifests.yaml_stream(docs, opts = {sep = "NOT A YAML SEPARATOR"})
