schema ProviderFamily:
    version: str
    marketplace: bool = True

providerFamily = lambda family: ProviderFamily -> ProviderFamily {
    family
}

v = providerFamily({
    version: "1.6.0"
})
