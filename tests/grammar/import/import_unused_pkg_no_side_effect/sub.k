__did_run = False

_sub_helper = lambda -> None {
    setattr(__dict__()["__did_run"], "_value", True)
}

_sub_helper()