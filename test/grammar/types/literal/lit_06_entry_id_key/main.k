schema Id:
    id?: int

schema Config:
    data?: {"A"|"B":Id}

c = Config {
    data = {
        A = Id()
        B = Id()
    }
}
