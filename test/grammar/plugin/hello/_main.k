import kcl_plugin.hello as hello

name = hello.tolower('KCL')

three = hello.add(1, 2)

a = hello.get_global_int()
hello.set_global_int(1)
b = hello.get_global_int()
