schema ProviderFamily:
    version: str
    marketplace: bool = True

providerFamily = lambda family: ProviderFamily -> ProviderFamily {
    {
        version: 1
    }
}

v = providerFamily({
    version: "1.6.0"
})
