config = {
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
