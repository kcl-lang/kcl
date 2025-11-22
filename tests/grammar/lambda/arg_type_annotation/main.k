import sub as s

identity: (s.Test) -> s.Test = lambda res: s.Test {
  res {
    name = "world" 
  }
}

c = identity(s.Test{name="hello"})
