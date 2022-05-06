schema Person:
    mixin [InfoMixin]
    name?: str
    age?: int
    info?: Info

    __settings__: {str:str} = {
        output_type = "IGNORE"
    }

schema Info:
    gender?: str
    meta?: {str:}

    __settings__: {str:str} = {
        output_type = "STANDALONE"
    }

schema InfoMixin:
    info |= Info {
        meta = {
            name = name if name else "feak",
            age = age if age else -1
        }
    }

person = "john"

alice = Person {
    name = "alice",
    age = 10,
    info: Info {gender = "girl"}
} if person == "alice" else Person {
    name = "john",
    info: Info {gender = "boy"}
}
