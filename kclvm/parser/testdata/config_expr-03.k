# test/grammar/attr_operator/config_inside/insert/dict_0/main.k

config = {
    main: {
        env: [
            {name: "ENV_1", value: "1"}
        ]
    }
    main: {
        env += [
            {name: "ENV_2", value: "2"}
        ]
    }
}
