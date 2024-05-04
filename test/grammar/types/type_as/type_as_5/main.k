import pkg
foo: pkg.Foo | pkg.Bar = pkg.Foo{}
bar = foo as pkg.Foo
