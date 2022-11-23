
# Stop on error.
set -e

# python3 path
python3_bin=`which python3`
kclvm_install_dir="$topdir/_build/dist/$os/kclvm"
pip_install_done_file="$kclvm_install_dir/lib/site-packages/kclvm.requirements.done.txt"

# check python3
if [ -z "$python3_bin" ]; then
    echo "python3 not found!"
    exit 1
fi

# once: pip install
if [ -f $pip_install_done_file ]; then
    exit 0
fi

# check python3 version
$python3_bin -c "import sys; sys.exit(0) if sys.version_info>=(3,7,3) else (print('please install python 3.7+') or sys.exit(1))"

# kclvm pip install all libs
$python3_bin -m pip install --upgrade pip
$python3_bin -m pip install kclvm
echo 'done' > $pip_install_done_file
