import manifests

test_replace_stream = lambda {
    manifests.yaml_stream(docs)
}

test_keep_stream = lambda {
    assert len(docs) == 2
}
