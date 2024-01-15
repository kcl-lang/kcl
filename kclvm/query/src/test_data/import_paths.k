import pkg
import pkg.pkg
import pkg.pkg as alias_pkg1
import pkg.pkg as alias_pkg2
import relative_pkg

schema Data:
    id?: int = 0
    value?: str = "value"

data = Data {value = "override_value"}

