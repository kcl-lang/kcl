
schema CheckAnyType:
    x_1?: any
    x_2?: any

    x_list_0?: any = None
    x_list_1?: [any]
    x_list_2?: [any]

    x_dict_v_1?: {str:any}
    x_dict_v_2?: {str:any}

x = CheckAnyType {
    x_1: 1
    x_2: 'abc'

    x_list_0: ['a']
    x_list_1: [1, 2, 3]
    x_list_2: ['a', 1, True]

    x_dict_v_1: {'a': 1}
    x_dict_v_2: {'a': 1, 'b':True}
}
