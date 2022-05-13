

schema CheckLiteralType:

    bool_0: True
    bool_1: False
    bool_2_1: True|False
    bool_2_2: True|False
    bool_3_1?: True|False
    bool_3_2?: True|False
    bool_3_3?: True|False

    int_0: 0
    int_1: 123
    int_2_1: 80|8080
    int_2_2: 80|8080

    float_p: 3.14
    float_e: 2.71
    float_e_p_1?: 3.14 | 2.71
    float_e_p_2?: 3.14 | 2.71
    float_e_p_3?: 3.14 | 2.71

    str_TCP: "TCP"
    str_UDP: "UDP"
    str_TCP_UCP_1?: "TCP" | "UDP"
    str_TCP_UCP_2?: "TCP" | "UDP"
    str_TCP_UCP_3?: "TCP" | "UDP"

all_lit_types = CheckLiteralType {
    bool_0: True
    bool_1: False
    bool_2_1: True
    bool_2_2: False
    bool_3_1: True
    bool_3_2: False
    bool_3_3: None

    int_0: 0
    int_1: 123
    int_2_1: 80
    int_2_2: 8080

    float_p: 3.14
    float_e: 2.71
    float_e_p_1: 3.14
    float_e_p_2: 2.71
    float_e_p_3: None

    str_TCP: "TCP"
    str_UDP: "UDP"
    str_TCP_UCP_1: "TCP"
    str_TCP_UCP_2: "UDP"
    str_TCP_UCP_3: None
}
