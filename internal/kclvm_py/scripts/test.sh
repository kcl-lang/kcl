#!/usr/bin/env bash
RED='\033[0;31m'
function red() {
  printf "${RED}$@${NC}\n"
}

if [ "$?" -ne 0 ]; then
  echo $(red update gitsubmodule failed! exit...)
  exit 1
fi

# Options
help_message=$(cat <<-END
    Usage:
        test.sh -h
            Print this help message
        test.sh -a [action]
            Perform a test
        test.sh
            Perform a test interactively
    Available actions:
        test_unit
            trigger unit test
        test_grammar
            trigger grammar test
        all
            trigger all tests
END
)
action=
while getopts "a:h:s:" opt; do
    case $opt in
        a)
        action="$OPTARG"
        ;;
        h)
        echo "$help_message"
        exit 1
        ;;
        \?) echo "Invalid option -$OPTARG"
        ;;
    esac
done

if [ "$action" == "" ]; then
    PS3='Please select the test scope: '
    options=("test_unit" "test_grammar" "all")
    select action in "${options[@]}"
    do
        case $action in
            "test_unit")
                $topdir/internal/kclvm_py/scripts/test_unit.sh
                break
                ;;
            "test_grammar")
                $topdir/internal/kclvm_py/scripts/test_grammar.sh
                break
                ;;
            "all")
                $topdir/internal/kclvm_py/scripts/test_unit.sh && $topdir/internal/kclvm_py/scripts/test_grammar.sh
                break
                ;;
            *) echo "Invalid action $REPLY:$action"
                exit 1
                break
                ;;
        esac
    done
fi
