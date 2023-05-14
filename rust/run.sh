#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"

_arg_input=("$@")

build() {
    cargo build --workspace || exit 1
}

word_count() {
    echo
    echo '=====  Word Count'

    timeout -k 2s 180s cargo run --bin mr-coordinator "${_arg_input[@]}" &
    pid=$!

    # allow coordinator time to start
    sleep 1

    # start some workers
    for _ in {1..4}; do
        timeout -k 2s 180s cargo run --bin mr-worker target/debug/libwc.so &
    done

    # wait for coordinator
    wait $pid

    # since workers are required to exit when a job is completely finished,
    # and not before, that means the job has finished.
    sort mr-out* | grep . > mmr-wc-all
    find . -maxdepth 1 -type f -name 'mr-*' -exec rm {} \;
    mv mmr-wc-all mr-wc-all

    # wait for remaining workers and coordinator to exit.
    wait

    echo "word count results in ${DIR}/mr-wc-all"
}

if [ ${#@} -eq 0 ]; then
    echo "Usage: run.sh inputfiles..."
    exit 1
fi

build
word_count
