schema User[arg: Arg]:
    name: str = "${arg.envName}"

schema Arg:
    envName: "staging" | "production"

_user = User({})
