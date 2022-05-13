import regex
schema Sample:
    foo: str  # Required, 不能为None, 且类型必须为str
    bar: int  # Required, 不能为None, 且类型必须为int
    fooList: [int]  # Required, 不能为None, 且类型必须为str列表
    color: "Red" | "Yellow" | "Blue"  # Required, 字面值联合类型，且必须为"Red", "Yellow", "Blue"中的一个，枚举作用
    id?: int  # Optional，可以留空，类型必须为int
    check:
        bar >= 0  # bar必须大于等于0
        bar < 100 # bar必须小于100
        len(fooList) > 0  # fooList不能为None，并且长度必须大于0
        len(fooList) < 100 # fooList不能为None，并且长度必须小于100
        regex.match(foo, "^The.*Foo$") # regex 正则表达式匹配
        bar in range(100) # range, bar范围只能为1到99
        bar in [2, 4, 6, 8] # enum, bar只能取2, 4, 6, 8
        bar % 2 == 0  # bar必须为2的倍数
        all foo in fooList {
            foo > 1
        }  # fooList中的所有元素必须大于1
        any foo in fooList {
            foo > 10
        }  # fooList中至少有一个元素必须大于10
        abs(id) > 10 if id is not None  # check if 表达式，当 id 不为空时，id的绝对值必须大于10
