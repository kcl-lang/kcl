schema ProviderFamily:
    [...str]: int
    version: str
    marketplace: bool = True

providerFamily = lambda family: ProviderFamily -> ProviderFamily {
    {
        **family
        version: 1
        hello: "world"
    }
}

v = providerFamily({
    version: 1
    hello: "world"
})
