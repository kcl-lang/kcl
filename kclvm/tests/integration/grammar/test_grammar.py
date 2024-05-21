"""This is a scripts to run KCL grammar test cases with the native target"""
import pytest
import os
import subprocess
import re
import yaml
import pathlib
from ruamel.yaml import YAML

TEST_FILE = "main.k"
STDOUT_GOLDEN = "stdout.golden"
STDERR_GOLDEN = "stderr.golden"
SETTINGS_FILE = "settings.yaml"
TEST_PATH = "test/grammar"

# Ruamel YAML instance
ruamel_yaml = YAML(typ="unsafe", pure=True)
# Convert None to null
ruamel_yaml.representer.add_representer(
    type(None),
    lambda dumper, data: dumper.represent_scalar(u"tag:yaml.org,2002:null", u"null"),
)


def find_test_dirs(path, category):
    result = []
    for root, dirs, files in os.walk(path + category):
        for name in files:
            if name == "main.k":
                result.append(root)
    return result


def compare_strings(result_strings: list, golden_strings: list):
    result = "\n".join(result_strings)
    golden_result = "\n".join(golden_strings)
    result_yaml_list = [r for r in list(ruamel_yaml.load_all(result)) if r]
    golden_yaml_list = [r for r in list(ruamel_yaml.load_all(golden_result)) if r]
    assert result_yaml_list == golden_yaml_list


def compare_results(result, golden_result):
    """Convert bytestring (result) and list of strings (golden_lines) both to
    list of strings with line ending stripped, then compare."""

    result_strings = result.decode().split("\n")
    golden_strings = golden_result.decode().split("\n")
    assert result_strings == golden_strings


def compare_results_with_lines(result, golden_lines):
    """Convert bytestring (result) and list of strings (golden_lines) both to
    list of strings with line ending stripped, then compare.
    """

    result_strings = result.decode().split("\n")
    golden_strings = []
    for line in golden_lines:
        clean_line = re.sub("\n$", "", line)
        golden_strings.append(clean_line)
    # List generated by split() has an ending empty string, when the '\n' is
    # the last character
    assert result_strings[-1] == "", "The result string does not end with a NEWLINE"
    golden_strings.append("")
    compare_strings(result_strings, golden_strings)


def generate_golden_file(py_file_name):
    if os.path.isfile(py_file_name):
        try:
            process = subprocess.Popen(
                ["kclvm", py_file_name],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=dict(os.environ),
            )
            stdout, stderr = process.communicate()
            assert (
                process.returncode == 0
            ), "Error executing file {}, exit code = {}".format(
                py_file_name, process.returncode
            )
        except Exception:
            raise
        return stdout
    return None


def read_settings_file(settings_file_name):
    if os.path.isfile(settings_file_name):
        try:
            with open(settings_file_name, "r") as stream:
                settings = yaml.safe_load(stream)
        except Exception:
            raise
        return settings
    return None


print("##### K Language Grammar Test Suite #####")
test_path = pathlib.Path(__file__).parent.parent.parent.parent.parent.joinpath(
    TEST_PATH
)
test_dirs = find_test_dirs(str(test_path), "")


def remove_ansi_escape_sequences(text):
    ansi_escape_pattern = re.compile(r'(?:\x1B[@-_]|[\x80-\x9F])[0-?]*[ -/]*[@-~]')
    return ansi_escape_pattern.sub('', text)


def remove_extra_empty_lines(text):
    lines = [line for line in text.splitlines() if line.strip()]
    return '\n'.join(lines)


@pytest.mark.parametrize("test_dir", test_dirs)
def test_grammar(test_dir):
    print("Testing {}".format(test_dir))
    test_settings = read_settings_file(os.path.join(test_dir, SETTINGS_FILE))
    kcl_command = ["kclvm_cli", "run", TEST_FILE]
    if test_settings and test_settings["kcl_options"]:
        kcl_command.extend(test_settings["kcl_options"].split())
    process = subprocess.Popen(
        kcl_command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=os.path.abspath(test_dir),
        env=dict(os.environ),
    )
    stdout, stderr = process.communicate()
    print("STDOUT:\n{}".format(stdout.decode()))
    print("STDERR:\n{}".format(stderr.decode()))
    # Attempt to use existing golden stdout.
    try:
        with open(
            os.path.join(test_dir, STDOUT_GOLDEN), "r"
        ) as golden_file:
            compare_results_with_lines(stdout, golden_file)
            assert process.returncode == 0
    except OSError:
        # Ignore when a golden file does not exist.
        pass
    except Exception:
        raise

    # Attempt to compare existing golden stdout.
    try:
        with open(
            os.path.join(test_dir, STDOUT_GOLDEN), "r"
        ) as golden_file:
            compare_results_with_lines(stdout, golden_file)
            assert process.returncode == 0
    except OSError:
        # Ignore when a golden file does not exist.
        pass
    except Exception:
        raise

    stderr_file = pathlib.Path(test_dir).joinpath(STDERR_GOLDEN)
    cwd = os.path.abspath(test_dir)
    if stderr_file.exists():
        golden = remove_extra_empty_lines(remove_ansi_escape_sequences(stderr_file.read_text()))
        stderr = remove_extra_empty_lines(remove_ansi_escape_sequences(stderr.decode()))
        golden = golden.replace("${CWD}", cwd)
        assert golden in stderr
        assert process.returncode > 0
