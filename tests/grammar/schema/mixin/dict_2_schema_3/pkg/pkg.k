schema Config:
    metadata: MetaData
    spec: Spec

schema MetaData:
    name: str

schema Spec:
    apps: [App]

schema App:
    spec: AppSpec

schema AppSpec:
    appName: str
    appType: str
