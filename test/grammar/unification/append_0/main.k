schema Main:
    env: [str]

schema Config:
    main: Main

config = Config {
    main: Main {
        env: ["s1"]
    }
}

config = Config {
    main: Main {
        env += ["s2"]
    }
}
