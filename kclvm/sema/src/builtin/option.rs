/// OptionHelp denotes all the option function calling usage.
#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct OptionHelp {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub default_value: String,
    pub help: String,
}

/// Print option helps to string
pub fn print_option_help(option_helps: &[OptionHelp]) -> String {
    let mut msg: String = "".to_string();

    // name=? (required) set name value
    // name=? (str,required) set name value
    // a=42 set a value
    // b=? set b value
    // obj=?
    // obj2=?

    msg.push_str("option list:\n");
    for opt in option_helps {
        let name = opt.name.clone();

        let default_value = if !opt.default_value.is_empty() {
            &opt.default_value
        } else {
            "?"
        };

        let s = format!("  -D {name}={default_value}");
        msg.push_str(s.as_str());

        // (required)
        // (str,required)
        if !opt.ty.is_empty() || opt.required {
            if opt.required && !opt.ty.is_empty() {
                let s = format!(" ({},{})", opt.ty, "required");
                msg.push_str(s.as_str());
            } else if !opt.ty.is_empty() {
                let s = format!(" ({})", opt.ty);
                msg.push_str(s.as_str());
            } else {
                msg.push_str(" (required)");
            }
        }

        if !opt.help.is_empty() {
            msg.push(' ');
            msg.push_str(opt.help.as_str());
        }

        msg.push('\n');
    }

    msg = msg.as_str().trim_end_matches('\n').to_string();
    msg
}
