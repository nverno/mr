#!/usr/bin/env bash

die() {
    local _ret="${2:-1}"
    test "${_PRINT_HELP:-no}" = yes && print_help >&2
    echo "$1" >&2
    exit "${_ret}"
}

begins_with_short_option() {
    local first_option all_short_options='rh'
    first_option="${1:0:1}"
    test "$all_short_options" = "${all_short_options/$first_option/}" && return 1 || return 0
}

_positionals=()
_arg_input=()
_arg_race="on"

print_help() {
    printf '%s\n' "Run mapreduce"
    printf 'Usage: %s [-r|--(no-)race] [-h|--help] [<input-1>] ... [<input-n>] ...\n' "$0"
    printf '\t%s\n' "<input>: input files for mapreducers"
    printf '\t%s\n' "-r, --race, --no-race: run with Go race detector (on by default)"
    printf '\t%s\n' "-h, --help: Prints help"
}

parse_commandline() {
    _positionals_count=0
    while test $# -gt 0
    do
	_key="$1"
	case "$_key" in
	    -r|--no-race|--race)
		_arg_race="on"
		test "${1:0:5}" = "--no-" && _arg_race="off"
		;;
	    -r*)
		_arg_race="on"
		_next="${_key##-r}"
		if test -n "$_next" -a "$_next" != "$_key"
		then
		    { begins_with_short_option "$_next" && shift && set -- "-r" "-${_next}" "$@"; } ||
                        die "The short option '$_key' can't be decomposed to ${_key:0:2} and -${_key:2}, because ${_key:0:2} doesn't accept value and '-${_key:2:1}' doesn't correspond to a short option."
		fi
		;;
	    -h|--help)
		print_help
		exit 0
		;;
	    -h*)
		print_help
		exit 0
		;;
	    *)
		_last_positional="$1"
		_positionals+=("$_last_positional")
		_positionals_count=$((_positionals_count + 1))
		;;
	esac
	shift
    done
}

assign_positional_args() {
    local _positional_name _shift_for=$1
    _positional_names=""
    _our_args=$((${#_positionals[@]} - 0))
    for ((ii = 0; ii < _our_args; ii++))
    do
	_positional_names="$_positional_names _arg_input[$((ii + 0))]"
    done

    shift "$_shift_for"
    for _positional_name in ${_positional_names}
    do
	test $# -gt 0 || break
	eval "$_positional_name=\${1}" || die "Error during argument parsing, possibly an Argbash bug." 1
	shift
    done
}


build() {
    go build "$RACE" mrcoordinator.go || exit 1
    go build "$RACE" mrworker.go || exit 1

    # Build mapreduce apps
    (cd ../mrapps/;
     find . -type f -name "*.go" -print0 | while IFS= read -r -d '' file
     do
         go build "$RACE" -buildmode=plugin "$file" || exit 1
     done) || exit 1
}

word_count() {
    echo
    echo '=====  Word Count'

    timeout -k 2s 180s ./mrcoordinator "${_arg_input[@]}" &
    pid=$!

    # allow coordinator time to create socket
    sleep 1

    # start some workers
    timeout -k 2s 180s ./mrworker ../mrapps/wc.so &
    timeout -k 2s 180s ./mrworker ../mrapps/wc.so &
    timeout -k 2s 180s ./mrworker ../mrapps/wc.so &

    # wait for coordinator
    wait $pid

    # since workers are required to exit when a job is completely finished,
    # and not before, that means the job has finished.
    sort mr-out* | grep . > mmr-wc-all
    rm mr-*
    mv mmr-wc-all mr-wc-all
    # wait for remaining workers and coordinator to exit.
    wait
}


parse_commandline "$@"
assign_positional_args 1 "${_positionals[@]}"

[ ${#_arg_input[@]} -eq 0 ] && die "Missing input files" 1
echo "Input files: ${_arg_input[*]}"

RACE=-race
[ "$_arg_race" = "off" ] && RACE=

build

word_count
