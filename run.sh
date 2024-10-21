#!/usr/bin/env bash

# Environment
getSystemInfo() {
    arch=$(uname -m)
    case $arch in
        armv7*) arch="arm";;
        aarch64) arch="arm64";;
        x86_64) arch="amd64";;
    esac

    os=$(echo `uname`|tr '[:upper:]' '[:lower:]')
}

getSystemInfo
topdir=$PWD
version=v$(cat VERSION)

# Options
help_message=$(cat <<-END
    Usage:
        run.sh -h
            Print this help message.
        run.sh -a [action]
            Perform an action.
        run.sh
            Perform an action interactively.
    Available actions:
        build
            Build the KCL package.
        release
            Create a releasing for the KCL package.
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
        s)
        sslpath="$OPTARG"
        ;;
        \?) echo "Invalid option -$OPTARG"
        ;;
    esac
done

if [ "$action" == "" ]; then
    PS3='Please select the action: '
    options=("build" "release")
    select action in "${options[@]}"
    do
        case $action in
            "build")
                break
                ;;
            "release")
                break
                ;;
            *) echo "Invalid action $REPLY:$action"
                exit 1
                break
                ;;
        esac
    done
fi

topdir=$topdir version=$version sslpath=$sslpath $topdir/scripts/$action.sh
