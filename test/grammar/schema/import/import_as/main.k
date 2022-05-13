import pkg1 as pkg

schema Another:
	my: pkg.MySchema

schema Son(pkg.MySchema):
	tt: str

a = Another{"my": {"s1": "asdf"}}

b = Son{"s1": "123x", "tt": "432e"}

c = pkg.MySchema{"s1": "asdf"}