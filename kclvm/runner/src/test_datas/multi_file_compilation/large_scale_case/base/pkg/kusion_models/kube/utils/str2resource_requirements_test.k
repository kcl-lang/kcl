schema Str2ResourceRequirementsCase[case]:
    value = case.value
    expected = case.expected
    result = Str2ResourceRequirements(value).result
    assert result == expected, "test failed expected ${expected} got ${result}"

schema TestStr2ResourceRequirements:
    cases = [
        {
            value = "cpu=100m,memory=100Mi,disk=1Gi"
            expected = {'requests': {'cpu': '100m', 'memory': '100Mi', 'ephemeral-storage': '1Gi'}, 'limits': {'cpu': '100m', 'memory': '100Mi', 'ephemeral-storage': '1Gi'}}
        }
        {
            value = "cpu=1,memory=2Gi,disk=20Gi"
            expected = {'requests': {'cpu': '1', 'memory': '2Gi', 'ephemeral-storage': '20Gi'}, 'limits': {'cpu': '1', 'memory': '2Gi', 'ephemeral-storage': '20Gi'}}
        }
    ]
    [
        Str2ResourceRequirementsCase(case) for case in cases
    ]
