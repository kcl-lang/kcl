import sys
from typing import List, Dict, Tuple
import json
from kclvm.tools.lint.reporters.base_reporter import BaseReporter
from kclvm.tools.lint.message.message import Message


LEVEL_MAP = {"E": "error", "W": "waring", "C": "note"}


class SARIFMeta:
    VERSION = "2.1.0"
    SCHEMA = "https://docs.oasis-open.org/sarif/sarif/v2.1.0/cs01/schemas/sarif-schema-2.1.0.json"
    NAME = "kcl-lint"
    KCLLINT_VERSION = "0.0.1"
    INFORMATION_URI = "https://kusion-docs.com/docs/reference/cli/kcl/lint"


class Rule:
    def __init__(self, id: str, default: str, short: str):
        self.id = id
        self.messageStrings = {
            "default": {"text": default},
            "shortStrings": {"text": short},
        }


class Result:
    def __init__(self, m: Message):
        self.ruleId = m.msg_id
        self.level = LEVEL_MAP[m.msg_id[0]] if (m.msg_id[0] in LEVEL_MAP) else "note"
        self.message = {"id": "default", "arguments": m.arguments}
        self.locations = [
            {
                "physicalLocation": {
                    "artifactLocation": {"uri": m.file},
                    "region": {"startLine": m.pos[0], "startColumn": m.pos[1]},
                }
            }
        ]


class Tool:
    def __init__(self, ids: List[str], MSGS: Dict[str, Tuple[str, str]]):
        self.driver = {
            "name": SARIFMeta.NAME,
            "version": SARIFMeta.KCLLINT_VERSION,
            "informationUri": SARIFMeta.INFORMATION_URI,
            "rules": [Rule(id, MSGS[id][2], MSGS[id][1]) for id in ids],
        }


class SarifLog(object):
    """Static Analysis Results Format (SARIF) Version 2.1.0 JSON Schema: a standard format for the output of static analysis tools."""

    def __init__(self, msgs: List[Message], MSGS: Dict[str, Tuple[str, str]]):
        self.version = SARIFMeta.VERSION
        # self.$schema = SARIFMeta.SCHEMA
        msg_ids = list(set([m.msg_id for m in msgs]))
        self.runs = [
            {"tool": Tool(msg_ids, MSGS), "results": [Result(m) for m in msgs]}
        ]


class SARIFReporter(BaseReporter):
    def __init__(self, linter, output=None, encoding=None):
        self.name = "sarif_reporter"
        self.output_file = linter.config.output_path
        super().__init__(linter, output, encoding)

    def print_msg(self, msgs: List[Message] = None):
        assert self.output_file
        sarif_log = SarifLog(msgs, self.linter.MSGS)
        json_str = json.dumps(
            sarif_log, default=lambda o: o.__dict__, sort_keys=True, indent=2
        )
        with open(self.output_file, "w") as f:
            current = sys.stdout
            sys.stdout = f
            print(json_str)
            sys.stdout = current
