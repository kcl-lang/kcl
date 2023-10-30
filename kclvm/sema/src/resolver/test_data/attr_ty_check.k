schema Data:
    id?: int

schema Config:
    datas: [Data]

config = Config {
    datas: [{id = "1"}]
}
