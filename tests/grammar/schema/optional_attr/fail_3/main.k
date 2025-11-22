schema App:
    data?: [int]
    version: Version

schema Version:
    versions: [str]

app = App {
    version = Version {}
}
