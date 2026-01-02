schema User[arg: Arg]:
    name: str = "${arg.envName}"

schema Arg:
    envName: "staging" | "production"

a = Arg {envName = "production"}
b = User(a)
