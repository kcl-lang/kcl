import pkg.pkg
import pkg
import .relative_pkg

schema Data:
    id?: int = 0
    value?: str = "value"

data = Data {
    value = "override_value"
}
