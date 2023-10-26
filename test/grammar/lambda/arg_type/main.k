import sub as s

identity = lambda res: s.Test {
  res {
    name = "world" 
  }
}

c = identity(s.Test{name="hello"})
