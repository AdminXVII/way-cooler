#!/usr/bin/env python2

import sys
import os
import re

from docopt import docopt

VERSION_REGEX = '(\\d+\\.\\d+\\.\\d+)'
BRANCH_REGEX = '$release*' + VERSION_REGEX + '^'
# If we grab the first 'version=' line in the Cargo files we'll be fine
CARGO_VERSION_LINE = '$version = "' + VERSION_REGEX + '"^'
README_CRATES_TAG = "/badge/crates\\.io/-v" + VERSION_REGEX

FILE_REGEX_MAP = {
    "Cargo.toml": CARGO_VERSION_LINE,
    "Cargo.lock": CARGO_VERSION_LINE,
    "README.md": README_CRATES_TAG
}

DOCOPT_USAGE = """way-cooler CI integration.

Usage:
  ci.py travis-check
  ci.py bump <old version> <new version> [-v]
  ci.py (-h | --help)
  ci.py --version

Options:
  -h --help    Show this menu
  -v           Be verbose, print actions taken
  --version    Show version information
"""

failed = False

def check_file_version(file_name, regex, expected):
    reg = re.compile(regex)
    with open(file_name) as f:
        for line in f.readlines():
            match = reg.match(line)
            if not match:
                continue
            elif match == expected:
                print('\t' + file_name + " updated.")
                return True
            else:
                print('\t' + file_name + ": expected " + expected + ", got " + match)
                return False

def check_release_branch(version):
    all_clear = True
    for (file_name, file_regex) in FILE_REGEX_MAP.items():
        print("Checking " + file_name)
        if not check_file_version(file_name, file_regex, version):
            all_clear = False
    return all_clear

if __name__ == "__main__":
    args = docopt(DOCOPT_USAGE, version="ci.py v1.0")
    if args["travis-check"]:
        travis_pr_branch = os.environ["TRAVIS_PULL_REQUEST_BRANCH"]
        if travis_pr_branch == "":
            print("Not running in a PR.")
            sys.exit(0)
        version_match = re.match(travis_pr_branch)
        if not version_match:
            print("Not in a release branch PR.")
            sys.exit(0)
        print("Checking versions in branch " + travis_pr_branch)
        if not check_release_branch(version_match):
            sys.stderr.write("Not all files matched!")
            sys.exit(2)

    elif args["bump"]:
        sys.stderr.write("Not supported yet")
        sys.exit(1)

    else:
        sys.stderr.write("Invalid arguments!\n")
        sys.exit(1)
