schema Config:
    cpu?: int

schema A:
    k?: Config

schema C(A):
    k: Config

a = C {
    k: Config {cpu: 1}
}