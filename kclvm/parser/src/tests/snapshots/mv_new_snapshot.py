#!/usr/bin/env python3
import os
import shutil
import pathlib

count = 0
dir = os.path.dirname(os.path.abspath(__file__))
for file in os.listdir(dir):
    file = os.path.join(dir, file)
    if file.endswith(".new"):
        to_file_name = file[0 : len(file) - len(".new")]
        print(f"to mv file {file} to {to_file_name}")
        shutil.move(file, to_file_name)
        print(f"moved file {file} to {to_file_name} done")
        count += 1

print(f"moved {count} files done")
