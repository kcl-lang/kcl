import base.pkg.models.app
import base.pkg.models.app.resource as res
import .values

# Note: This is a sample configuration file. Please modify it according to your needs.
# Note：
#   1. This file is used to configure the application. 
#   2. The configuration file is written in KCL.

c = {
    "a": 1
}

_access = test.ServiceAccess {
    iType = "kkkkkkk"
    sType = dsType
    TestStr = ["${test_str}"]
    ports = [80, 443]
    booltest = True
}

_access3 = test.ServiceAccess {
    iType = "kkkkkkk"
    sType = dsType
    TestStr = ["${test_str}"]
    ports = [80, 443]
    booltest = True
}