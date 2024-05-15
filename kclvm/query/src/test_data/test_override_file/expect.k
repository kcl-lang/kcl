import base.pkg.kusion_models.app
import base.pkg.kusion_models.app.vip as vip
import base.pkg.kusion_models.app.container
import base.pkg.kusion_models.app.resource as res
import base.pkg.kusion_models.app.sidecar
import base.pkg.models.app
import base.pkg.models.app.resource as res
import .values

# Note: This is a sample configuration file. Please modify it according to your needs.
# Note：
#   1. This file is used to configure the application. 
#   2. The configuration file is written in KCL.
c = {
    "a": 1
    b = {"a": "b"}
}
_access = test.ServiceAccess {
    iType = "kkkkkkk"
    sType = dsType
    TestStr = ["${test_str}"]
    ports = [80, 443]
    booltest = True
    mergedattr = 1
    a: {b: {c = 2}, c: {b = 3}}
}

_access3 = test.ServiceAccess {
    iType = "kkkkkkk"
    sType = dsType
    TestStr = ["${test_str}"]
    ports = [80, 443]
    booltest = True
}

b = {"c": 2}
d = {
    e: {
        f: {g = 3}
    }
}
_access4 = test.ServiceAccess {
    iType = "kkkkkkk"
    sType = dsType
    TestStr = ["${test_str}"]
    ports = [80, 443]
    booltest = True
}

_access5 = {iType = "dddddd"}
a = b
_access6 = "a6"
