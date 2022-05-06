schema Main:
    env: [str]

main = Main {
    env: ["s1"]
}

main = Main {
    env += ["s2"]
}
