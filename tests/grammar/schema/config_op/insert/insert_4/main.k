schema Spec:
    counts: [int]

schema Data:
    spec?: Spec

d1 = Data {
    spec.counts += [0]
}
d2 = d1 {
    spec.counts += [1]
}
