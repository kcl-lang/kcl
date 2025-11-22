schema Data:
    name: str
    type: str

data = {
    name = "data"
}

datas: {str:{str:Data}} = {
    data.data = data
}
