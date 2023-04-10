"""
this testing framework is developed based on pytest.
see quick start of pytest: https://docs.pytest.org/en/latest/example/simple.html

"""
import os
import subprocess
from pathlib import Path

import pytest
from ruamel.yaml import YAML
from collections.abc import Mapping, Sequence

TEST_FILE = "kcl.yaml"
CI_TEST_DIR = "ci-test"
STDOUT_GOLDEN = "stdout.golden.yaml"
SETTINGS_FILE = "settings.yaml"

ROOT_STR = "konfig"
ROOT = str(Path(__file__).parent.joinpath(ROOT_STR))

yaml = YAML(typ="unsafe", pure=True)


def find_test_dirs():
    result = []
    root_dirs = [ROOT]
    for root_dir in root_dirs:
        for root, _, files in os.walk(root_dir):
            for name in files:
                if name == TEST_FILE:
                    result.append(root)
    return result


def compare_results(result, golden_result):
    """Convert result and golden_result string to string lines with line ending stripped, then compare."""
    result = [
        r
        for r in list(yaml.load_all(result))
        if r and r.get("kind") != "SecretProviderClass"
    ]
    # Convert kusion compile spec to kcl result
    expected = [
        r
        for r in list(yaml.load_all(golden_result))[0]
        if r["attributes"]
        # Remove CRDs
        and not r["id"].startswith("apiextensions.k8s.io/v1:CustomResourceDefinition")
    ]
    print(len(result), len(expected))
    assert compare_unordered_yaml_objects(result, expected)


def compare_unordered_yaml_objects(result, golden_result):
    """Comparing the contents of two YAML objects for equality in an unordered manner"""
    if isinstance(result, Mapping) and isinstance(golden_result, Mapping):
        if result.keys() != golden_result.keys():
            return False
        for key in result.keys():
            if not compare_unordered_yaml_objects(result[key], golden_result[key]):
                return False

        return True
    elif isinstance(result, Sequence) and isinstance(golden_result, Sequence):
        if len(result) != len(golden_result):
            return False
        for item in result:
            if item not in golden_result:
                return False
        for item in golden_result:
            if item not in result:
                return False
        return True
    else:
        return result == golden_result


def has_settings_file(directory):
    settings_file = directory / SETTINGS_FILE
    return settings_file.is_file()


print("##### K Language Grammar Test Suite #####")
test_dirs = find_test_dirs()
pwd = str(Path(__file__).parent.parent.parent)
os.environ["PYTHONPATH"] = pwd


@pytest.mark.parametrize("test_dir", test_dirs)
def test_konfigs(test_dir):
    print(f"Testing {test_dir}")
    test_dir = Path(test_dir)
    kcl_file_name = test_dir / TEST_FILE
    ci_test_dir = test_dir / CI_TEST_DIR
    if not ci_test_dir.is_dir():
        # Skip invalid test cases
        return
    golden_file = ci_test_dir / STDOUT_GOLDEN
    if not golden_file.is_file():
        # Skip invalid test cases
        return
    kcl_command = ["kcl"]
    if has_settings_file(ci_test_dir):
        kcl_command.append("-Y")
        kcl_command.append(f"{CI_TEST_DIR}/{SETTINGS_FILE}")
        kcl_command.append(f"kcl.yaml")
    else:
        kcl_command.append(f"{TEST_FILE}")
    process = subprocess.run(
        kcl_command, capture_output=True, cwd=test_dir, env=dict(os.environ)
    )
    stdout, stderr = process.stdout, process.stderr
    print(f"STDOUT:\n{stdout.decode()}")
    assert (
        process.returncode == 0 and len(stderr) == 0
    ), f"Error executing file {kcl_file_name}.\nexit code = {process.returncode}\nstderr = {stderr}"
    if process.returncode == 0 and len(stderr) == 0:
        try:
            with open(golden_file, "r") as golden:
                compare_results(stdout.decode(), golden)
        except FileNotFoundError:
            raise Exception(f"Error reading expected result from file {golden_file}")
