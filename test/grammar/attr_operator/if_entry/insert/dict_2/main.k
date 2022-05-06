schema Main:
    env: [{str:str}]

schema Config:
    main: Main

config = Config {
    main: {
        env: [
            {name: "ENV_1", value: "1"}
        ]
    }
    main: {
        if True:
            env += [
                {name: "ENV_2", value: "2"}
            ]
    }
}
