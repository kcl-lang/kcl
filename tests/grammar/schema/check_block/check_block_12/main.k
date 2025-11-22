schema Info:
    msg: str

schema Person:
    info: Info = {
        "msg": "message"
    }
    check:
        info.msg, "{}".format(info.msg)

person = Person {}
