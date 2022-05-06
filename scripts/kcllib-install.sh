
# kclvm path
kclvm=$topdir/_build/dist/$os/kclvm/bin/kclvm
install_list=$topdir/scripts/requirements.txt

# kclvm pip install all libs
$kclvm -m pip install -r $install_list 
