#! /usr/bin/env python3

import kclvm.config


def write_out(inputs):
    outputs = inputs
    if kclvm.config.output:
        with open(kclvm.config.output, "w") as f:
            f.write(outputs)
    else:
        print(outputs, end="")


LOG_INDENT_STRING = "    "
log_indent = 0


def write_log(message, level=1):
    """Write log message whose level is no less than the verbosity level"""
    if kclvm.config.verbose >= level:
        for _ in range(log_indent):
            print(LOG_INDENT_STRING, end="")
        print(message)


def indent_log(step=1):
    """Adjust the indentation level of log messages"""
    global log_indent
    log_indent += step
