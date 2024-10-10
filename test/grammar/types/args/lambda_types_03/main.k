schema Cluster:
    name: str
    extra: int

a: Cluster = { name = "abc", extra = 6 }

enrich = lambda value {
    [a]
}

result_typesafe: [Cluster] = enrich({name = "abc", extra = 6 })
