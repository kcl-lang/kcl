cd %~dp0

python3 -m pip install --upgrade pip
python3 -m pip install kclvm pytest pytest-xdist
python3 -m pytest -vv -n 10
