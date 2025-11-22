import regex

regex_source = "Apple,Google,Baidu,Xiaomi"
regex_split = regex.split(regex_source, ",")
regex_replace = regex.replace(regex_source, ",", "|")
regex_compile = regex.compile("$^")
regex_search = regex.search("aaaa", "a")
regex_find_all = regex.findall("aaaa", "a")
regex_result = regex.match("192.168.0.1", "^(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|[1-9])\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)$")
regex_result_false = regex.match("192.168.0,1", "^(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|[1-9])\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)\\."+"(1\\d{2}|2[0-4]\\d|25[0-5]|[1-9]\\d|\\d)$")
