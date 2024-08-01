schema ProviderFamily:
    version: str
    marketplace: bool = True

providerFamily = lambda family: ProviderFamily -> ProviderFamily {
    {}
}

v = providerFamily({
    version: "1.6.0"
    hello: "world"
})
