config = {
    main: {
        env = [
            {name: "ENV_1", value: "1"}
        ]
    }
    if False:
        main: {
            env = [
                {name: "ENV_2", value: "2"}
            ]
        }
    else:
        main: {
            env = [
                {name: "ENV_3", value: "3"}
            ]
        } 
}
