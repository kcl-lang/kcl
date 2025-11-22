import file

contents = [ file.read(_x) for _x in file.glob("./*.json")]
