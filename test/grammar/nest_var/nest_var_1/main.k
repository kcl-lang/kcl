schema KeyVal:
    labels?: {str:}

schema Son:
    metadata?: {str:} | str
    name?: str = None
    keyVals?: KeyVal

son = Son {
    metadata.labels.key: 123,
    keyVals: {
        labels.key: 456
    }
    keyVals.labels.key2: 789
}
