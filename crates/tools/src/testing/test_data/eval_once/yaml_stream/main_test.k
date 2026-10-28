import manifests

test_good_stream = lambda {
    manifests.yaml_stream(docs)
}

test_bad_stream = lambda {
    manifests.yaml_stream(docs, opts = {sep = "NOT A YAML SEPARATOR"})
}

test_after_bad_stream = lambda {
    assert len(docs) == 2
}
