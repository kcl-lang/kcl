import pkg

schema Data:  
    a = pkg.a
    aStr = "${pkg.a}"
    config = {
        "${pkg.a}": "1"
    }

data = Data {}
