schema ProviderFamily:
    version: str
    marketplace: bool = True

providerFamily = lambda -> ProviderFamily {
    {
        version: "1.6.0"
    }
}

v = providerFamily()
