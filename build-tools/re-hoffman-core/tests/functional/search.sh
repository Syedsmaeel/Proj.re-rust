#!/usr/bin/env bash

source common.sh

clearStoreIfPossible
clearCache

(( $(hoffman search -f search.hoffman '' hello | wc -l) > 0 ))

# Check descriptions are searched
(( $(hoffman search -f search.hoffman '' broken | wc -l) > 0 ))

# Check search that matches nothing
(( $(hoffman search -f search.hoffman '' nosuchpackageexists | wc -l) == 0 ))

# Search for multiple arguments
(( $(hoffman search -f search.hoffman '' hello empty | wc -l) == 2 ))

# Multiple arguments will not exist
(( $(hoffman search -f search.hoffman '' hello broken | wc -l) == 0 ))

# No regex should return an error
(( $(hoffman search -f search.hoffman '' | wc -l) == 0 ))

## Search expressions

# Check that empty search string matches all
hoffman search -f search.hoffman '' ^ | grepQuiet foo
hoffman search -f search.hoffman '' ^ | grepQuiet bar
hoffman search -f search.hoffman '' ^ | grepQuiet hello

## Tests for multiple regex/match highlighting

e=$'\x1b' # grep doesn't support \e, \033 or even \x1b
# Multiple overlapping regexes
(( $(hoffman search -f search.hoffman '' 'oo' 'foo' 'oo' | grep -c "$e\[32;1mfoo$e\\[0;1m") == 1 ))
(( $(hoffman search -f search.hoffman '' 'broken b' 'en bar' | grep -c "$e\[32;1mbroken bar$e\\[0m") == 1 ))

# Multiple matches
# Searching for 'o' should yield the 'o' in 'broken bar', the 'oo' in foo and 'o' in hello
(( $(hoffman search -f search.hoffman '' 'o' | grep -Eoc "$e\[32;1mo{1,2}$e\[(0|0;1)m") == 3 ))
# Searching for 'b' should yield the 'b' in bar and the two 'b's in 'broken bar'
# NOTE: This does not work with `grep -c` because it counts the two 'b's in 'broken bar' as one matched line
(( $(hoffman search -f search.hoffman '' 'b' | grep -Eo "$e\[32;1mb$e\[(0|0;1)m" | wc -l) == 3 ))

## Tests for --exclude
(( $(hoffman search -f search.hoffman ^ -e hello | grep -c hello) == 0 ))

(( $(hoffman search -f search.hoffman foo ^ --exclude 'foo|bar' | grep -Ec 'foo|bar') == 0 ))
(( $(hoffman search -f search.hoffman foo ^ -e foo --exclude bar | grep -Ec 'foo|bar') == 0 ))
[[ $(hoffman search -f search.hoffman '' ^ -e bar --json | jq -c 'keys') == '["foo","hello"]' ]]
