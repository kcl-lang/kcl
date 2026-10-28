import manifests

# The same output, planned with hidden values shown.
test_show_hidden = lambda {
    manifests.yaml_stream([{}], opts = {ignore_private = False})
}

test_after_show_hidden = lambda {
    assert _x == 2
}
