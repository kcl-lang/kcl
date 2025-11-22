schema Config:
    main: {str:[{str:str}]} = {env: [{"ENV0": "0"}]}

config = Config {
    main: {
        env: [
            {"ENV1": "1"}
        ]
    }
    main: {
        env: [
            {"ENV2": "2"}
        ]
        env: [
            {"ENV3": "3"}
        ]
    }
    main.env: [
        {"ENV4": "4"}
    ]
}
