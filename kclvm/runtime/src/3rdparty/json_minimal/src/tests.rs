use super::*;

#[test]
fn test_make_json() {
    let mut json = Json::new();

    let greeting = Json::OBJECT {
        name: String::from("Greeting"),

        value: Box::new(Json::STRING(String::from("Hello, world!"))),
    };

    json.add(greeting);

    let mut days_in_the_week = Json::OBJECT {
        name: String::from("Days in the week"),

        value: Box::new(Json::JSON(Vec::new())),
    };

    let mut days = Json::ARRAY(Vec::new());

    days.add(Json::STRING(String::from("Monday")))
        .add(Json::STRING(String::from("Tuesday")))
        .add(Json::STRING(String::from("Wednesday")))
        .add(Json::STRING(String::from("Thursday")))
        .add(Json::STRING(String::from("Friday")))
        .add(Json::STRING(String::from("Saturday")))
        .add(Json::STRING(String::from("Sunday")));

    days_in_the_week
        .add(Json::OBJECT {
            name: String::from("Total number of days"),

            value: Box::new(Json::NUMBER(7.0)),
        })
        .add(Json::OBJECT {
            name: String::from("They are called"),

            value: Box::new(days),
        });

    json.add(days_in_the_week);

    let mut conclusion = Json::OBJECT {
        name: String::from("Conclusion"),

        value: Box::new(Json::JSON(Vec::new())),
    };

    conclusion
        .add(Json::OBJECT {
            name: String::from("Minimal in my opinion"),

            value: Box::new(Json::BOOL(true)),
        })
        .add(Json::OBJECT {
            name: String::from("How much I care about your opinion"),

            value: Box::new(Json::NULL),
        })
        .add(Json::OBJECT {
            name: String::from("Comment"),

            value: Box::new(Json::STRING(String::from(";)"))),
        });

    json.add(conclusion);

    assert_eq!(
            "{\"Greeting\":\"Hello, world!\",\"Days in the week\":{\"Total number of days\":7,\"They are called\":[\"Monday\",\"Tuesday\",\"Wednesday\",\"Thursday\",\"Friday\",\"Saturday\",\"Sunday\"]},\"Conclusion\":{\"Minimal in my opinion\":true,\"How much I care about your opinion\":null,\"Comment\":\";)\"}}",
            &json.print()
        )
}

#[test]
fn test_get_mut() {
    let mut json = Json::new();

    json.add(Json::OBJECT {
        name: String::from("Greeting"),

        value: Box::new(Json::STRING(String::from("Hello, world!"))),
    });

    match json.get_mut("Greeting") {
        Some(json) => match json {
            Json::OBJECT { name: _, value } => match value.unbox_mut() {
                Json::STRING(val) => {
                    assert_eq!("Hello, world!", val);

                    val.push_str(" How are you?");

                    assert_eq!("Hello, world! How are you?", val);
                }
                _ => {
                    panic!("Expected `Json::STRING`!!!");
                }
            },
            _ => {
                panic!("Expected `Json::OBJECT`!!!");
            }
        },
        None => {
            panic!("Not found!!!");
        }
    }
}

#[test]
fn test_parse_number() {
    let mut incr: usize = 0;

    match Json::parse_number(b"36.36", &mut incr, &ParseOption::default()) {
        Ok(json) => match json {
            Json::NUMBER(val) => {
                assert_eq!(val, 36.36);
            }
            json => {
                panic!("Expected Json::NUMBER but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }
}

#[test]
fn test_parse_bool() {
    let mut incr: usize = 0;

    match Json::parse_bool(b"true", &mut incr, &ParseOption::default()) {
        Ok(json) => match json {
            Json::BOOL(val) => {
                assert_eq!(val, true);
            }
            json => {
                panic!("Expected Json::BOOL but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }

    incr = 0;

    match Json::parse_bool(b"false", &mut incr, &ParseOption::default()) {
        Ok(json) => match json {
            Json::BOOL(val) => {
                assert_eq!(val, false);
            }
            json => {
                panic!("Expected Json::BOOL but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }
}

#[test]
fn test_parse_null() {
    let mut incr: usize = 0;

    match Json::parse_null(b"null", &mut incr, &ParseOption::default()) {
        Ok(json) => match json {
            Json::NULL => {}
            json => {
                panic!("Expected Json::NULL but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }
}

#[test]
fn test_parse_array() {
    let mut incr: usize = 0;

    match Json::parse_array(
        b"[1,\"two\",true,[\"array\",[\"another one\",[\"another one\",1.5]]]]",
        &mut incr,
        &ParseOption::default(),
    ) {
        Ok(json) => match json {
            Json::ARRAY(vals) => {
                assert_eq!(vals.len(), 4);
            }
            json => {
                panic!("Expected Json::ARRAY but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }
}

#[test]
fn test_parse_json() {
    let mut incr: usize = 0;

    match Json::parse_json(b"{\"on\",\"off\"}", &mut incr, &ParseOption::default()) {
        Ok(json) => match json {
            Json::JSON(vals) => {
                assert_eq!(vals.len(), 2);
            }
            json => {
                panic!("Expected Json::ARRAY but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }
}

#[test]
fn test_parse_json_2() {
    let mut incr: usize = 0;

    match Json::parse_json(
        b"{\"on\",\"off\",\"OBJECT\":{\"ARRAY\":[\"on\",\"off\"]},\"on or off?\"}",
        &mut incr,
        &ParseOption::default(),
    ) {
        Ok(json) => match json {
            Json::JSON(vals) => {
                assert_eq!(vals.len(), 4);
            }
            json => {
                panic!("Expected Json::ARRAY but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }
}

#[test]
fn test_parse_object() {
    let mut incr: usize = 0;

    match Json::parse_string(b"\"String\":\"Value\"", &mut incr, &ParseOption::default()) {
        Ok(json) => match json {
            Json::OBJECT { name, value } => {
                assert_eq!(name, "String");

                match value.unbox() {
                    Json::STRING(val) => {
                        assert_eq!(val, "Value");
                    }
                    json => {
                        panic!("Expected Json::STRING but found {:?}", json);
                    }
                }
            }
            json => {
                panic!("Expected Json::OBJECT but found {:?}", json);
            }
        },
        Err(e) => {
            parse_error(e);
        }
    }
}

#[test]
fn test_parse() {
    match Json::parse(b"{\"Greeting\":\"Hello, world!\",\"Days in the week\":{\"Total number of days\":7,\"They are called\":[\"Monday\",\"Tuesday\",\"Wednesday\",\"Thursday\",\"Friday\",\"Saturday\",\"Sunday\"]},\"Minimal in my opinion\":true,\"How much I care about your opinion\":null}") {
            Ok(json) => {
                match json {
                    Json::JSON(values) => {
                        assert_eq!(values.len(),4);

                        match &values[0] {
                            Json::OBJECT { name, value } => {
                                assert_eq!("Greeting",name);

                                match value.unbox() {
                                    Json::STRING(val) => {
                                        assert_eq!("Hello, world!",val);
                                    },
                                    json => {
                                        panic!("Expected Json::STRING but found {:?}",json);
                                    }
                                }
                            },
                            json => {
                                panic!("Expected Json::OBJECT but found {:?}",json);
                            }
                        }

                        match &values[1] {
                            Json::OBJECT { name, value } => {
                                assert_eq!("Days in the week",name);

                                match value.unbox() {
                                    Json::JSON(values) => {
                                        assert_eq!(values.len(),2);

                                        match &values[0] {
                                            Json::OBJECT { name, value } => {
                                                assert_eq!("Total number of days",name);

                                                match value.unbox() {
                                                    Json::NUMBER(num) => {
                                                        assert_eq!(*num,7.0);
                                                    },
                                                    json => {
                                                        panic!("Expected Json::NUMBER but found {:?}",json);
                                                    }
                                                }
                                            },
                                            json => {
                                                panic!("Expected Json::OBJECT but found {:?}",json);
                                            }
                                        }

                                        match &values[1] {
                                            Json::OBJECT { name, value } => {
                                                assert_eq!("They are called",name);

                                                match value.unbox() {
                                                    Json::ARRAY(vals) => {
                                                        assert_eq!(vals.len(),7);

                                                        for n in 0..7 {
                                                            match &vals[n] {
                                                                Json::STRING(val) => {
                                                                    match val.as_bytes() {
                                                                        b"Monday" => {

                                                                        },
                                                                        b"Tuesday" => {

                                                                        },
                                                                        b"Wednesday" => {

                                                                        },
                                                                        b"Thursday" => {

                                                                        },
                                                                        b"Friday" => {

                                                                        },
                                                                        b"Saturday" => {

                                                                        },
                                                                        b"Sunday" => {

                                                                        },
                                                                        d => {
                                                                            panic!("\"{:?}\" is not a day of the week!!",d);
                                                                        }
                                                                    }
                                                                },
                                                                json => {
                                                                    panic!("Expected Json::STRING but found {:?}",json);
                                                                }
                                                            }
                                                        }
                                                    },
                                                    json => {
                                                        panic!("Expected Json::ARRAY but found {:?}",json);
                                                    }
                                                }
                                            },
                                            json => {
                                                panic!("Expected Json::OBJECT but found {:?}",json);
                                            }
                                        }
                                    },
                                    json => {
                                        panic!("Expected Json::JSON but found {:?}",json);
                                    }
                                }
                            },
                            json => {
                                panic!("Expected Json::OBJECT but found {:?}",json);
                            }
                        }
                    },
                    json => {
                        panic!("Expected Json::JSON but found {:?}",json);
                    }
                }
            },
            Err(e) => {
                parse_error(e);
            }
        }
}

#[test]
fn test_parse_2() {
    #[allow(unused_assignments)]

        let json = match Json::parse(b"{\"Greeting\":\"Hello, world!\",\"Days of the week\":{\"Total number of days\":7,\"They are called\":[\"Monday\",\"Tuesday\",\"Wednesday\",\"Thursday\",\"Friday\",\"Saturday\",\"Sunday\"]},\"Conclusion\":{\"Minimal in my opinion\":true,\"How much I care about your opinion\":null,\"Comment\":\";)\"}}") {
            Ok(json) => {
                json
            },
            Err( (position,message) ) => {
                panic!("`{}` at position `{}`!!!",message,position);
            }
        };

    match json.get("Greeting") {
        Some(json) => match json {
            Json::OBJECT { name: _, value } => match value.unbox() {
                Json::STRING(val) => {
                    assert_eq!("Hello, world!", val);
                }
                json => {
                    panic!("Expected Json::STRING but found {:?}", json);
                }
            },
            json => panic!("Expected Json::JSON but found {:?}!!!", json),
        },
        None => {
            panic!("Couln't find Greeting. How rude!");
        }
    }

    match json.get("Days of the week") {
        // Hint: You can also use `get_mut` to aid in editing/creating jsons...
        Some(json) => match json {
            Json::OBJECT { name: _, value } => match value.unbox() {
                Json::JSON(values) => {
                    assert_eq!(values.len(), 2);

                    match &values[0] {
                        Json::OBJECT { name, value: _ } => {
                            assert_eq!("Total number of days", name);
                        }
                        json => {
                            panic!("Expected Json::OBJECT but found {:?}!!!", json);
                        }
                    }

                    match &values[1] {
                        Json::OBJECT { name, value: _ } => {
                            assert_eq!("They are called", name);
                        }
                        json => {
                            panic!("Expected Json::OBJECT but found {:?}!!!", json);
                        }
                    }
                }
                json => {
                    panic!("Expected Json::JSON but found {:?}!!!", json);
                }
            },
            json => {
                panic!("Expected Json::OBJECT but found {:?}!!!", json);
            }
        },
        None => {
            panic!("Days of the week not found!");
        }
    }
}

#[test]
fn parse_strange() {
    let json = match Json::parse(b"[0,{\"hello\":\"world\",\"what's\":\"up?\"}]") {
        Ok(json) => json,
        Err((pos, msg)) => {
            panic!("`{}` at position {}", msg, pos);
        }
    };

    match json {
        Json::ARRAY(vals) => {
            assert_eq!(vals.len(), 2);

            match &vals[0] {
                Json::NUMBER(n) => {
                    assert_eq!(*n, 0.0);
                }
                json => {
                    panic!("Expected Json::NUMBER but found {:?}!!!", json);
                }
            }

            match &vals[1] {
                Json::JSON(vals) => {
                    assert_eq!(2, vals.len());

                    match &vals[0] {
                        Json::OBJECT { name, value: _ } => {
                            assert_eq!("hello", name);
                        }
                        json => {
                            panic!("Expected Json::ARRAY but found {:?}!!!", json);
                        }
                    }

                    match &vals[1] {
                        Json::OBJECT { name, value: _ } => {
                            assert_eq!("what's", name);
                        }
                        json => {
                            panic!("Expected Json::ARRAY but found {:?}!!!", json);
                        }
                    }
                }
                json => {
                    panic!("Expected Json::JSON but found {:?}!!!", json);
                }
            }
        }
        json => {
            panic!("Expected Json::ARRAY but found {:?}!!!", json);
        }
    }
}

#[test]
fn parse_escape_sequence() {
    let json = match Json::parse(br#""a \" \/ \b \f \n \r \t \u2764 z""#) {
        Ok(json) => json,
        Err((pos, msg)) => {
            panic!("`{}` at position {}", msg, pos);
        }
    };

    match json {
        Json::STRING(string) => {
            assert_eq!(string, "a \" / \u{8} \u{c} \n \r \t ❤ z");
        }
        json => {
            panic!("Expected Json::STRING but found {:?}!!!", json);
        }
    }
}

#[test]
fn parse_escape_sequence_in_array() {
    let json = match Json::parse(br#"["\"foo"]"#) {
        Ok(json) => json,
        Err((pos, msg)) => {
            panic!("`{}` at position {}", msg, pos);
        }
    };

    match json {
        Json::ARRAY(vals) => {
            assert_eq!(vals.len(), 1);

            match &vals[0] {
                Json::STRING(n) => {
                    assert_eq!(*n, "\"foo");
                }
                json => {
                    panic!("Expected Json::STRING but found {:?}!!!", json);
                }
            }
        }
        json => {
            panic!("Expected Json::ARRAY but found {:?}!!!", json);
        }
    }
}

#[test]
fn parse_non_ascii() {
    let json = match Json::parse(r#""a ❤ z""#.as_bytes()) {
        Ok(json) => json,
        Err((pos, msg)) => {
            panic!("`{}` at position {}", msg, pos);
        }
    };

    match json {
        Json::STRING(string) => {
            assert_eq!(string, "a ❤ z");
        }
        json => {
            panic!("Expected Json::STRING but found {:?}!!!", json);
        }
    }
}

#[test]
fn parse_pretty() {
    let json = match Json::parse(b"{\r\n\t\"Array\": [\r\n\t\t\"First\" ,\r\n\r\n\t\t2 ,\r\n\r\n\t\t[\"Three\"] ,\r\n\r\n\t\t3.6\r\n\t],\r\n\t{\r\n\r\n\t\t\"Sub-Object\": \"Hello, world!\"\r\n\t}\r\n}") {
        Ok(json) => json,
        Err((pos, msg)) => {
            panic!("`{}` at position {}", msg, pos);
        }
    };

    match json {
        Json::JSON(values) => {
            match values[0].unbox() {
                Json::OBJECT { name, value } => {
                    assert_eq!(name, "Array");

                    match value.unbox() {
                        Json::ARRAY(values) => {
                            assert_eq!(values.len(), 4);
                        }
                        json => {
                            panic!("Expected Json::ARRAY but found {:?}!!!", json);
                        }
                    }
                }
                json => {
                    panic!("Expected Json::OBJECT but found {:?}!!!", json);
                }
            }

            match values[1].unbox() {
                Json::JSON(values) => match values[0].unbox() {
                    Json::OBJECT { name, value } => {
                        assert_eq!(name, "Sub-Object");

                        match value.unbox() {
                            Json::STRING(value) => {
                                assert_eq!(value, "Hello, world!");
                            }
                            json => {
                                panic!("Expected Json::STRING but found {:?}!!!", json);
                            }
                        }
                    }
                    json => {
                        panic!("Expected Json::OBJECT but found {:?}!!!", json);
                    }
                },
                json => {
                    panic!("Expected Json::Json but found {:?}!!!", json);
                }
            }
        }
        json => {
            panic!("Expected Json::JSON but found {:?}!!!", json);
        }
    }
}

fn parse_error((pos, msg): (usize, &str)) {
    panic!("`{}` at position `{}`!!!", msg, pos);
}
