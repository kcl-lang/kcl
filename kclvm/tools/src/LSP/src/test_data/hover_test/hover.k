schema Person:
    """
    hover doc test 

    Attributes
    ----------
    name : str, default is False, required
        name doc test
    age : int, default is False, optional
        age doc test
  
    """
    name: str
    age?: int
    
p = Person{
    name: "Alice"
    age: 1
}

import base64
abdc = base64.encode("1")
abcd = "a".count()

print(1)
