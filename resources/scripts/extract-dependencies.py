#!/bin/env python
import os
import subprocess
from pathlib import Path

ROOT_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
BIN_DIR = ROOT_DIR / "target" / "release"
BINARIES = []

for bin_name in ["git-flow-rs"]:
    path = BIN_DIR / bin_name
    if path.exists():
        BINARIES.append(path)

for binary in BINARIES:
    dependencies = set()
    print(os.path.basename(str(binary)))
    output = subprocess.check_output(["ldd", str(binary)]).decode().splitlines()
    for line in output:
        if " => " in line:
            so_file = line.split(" => ")[1].split(" (")[0]
            p_output = subprocess.check_output(
                ["pacman", "-Qo", so_file], env={"LANG": "C"}
            ).decode()
            dependencies.add(p_output.split(" ")[-2])
    dependencies = sorted(list(dependencies))
    for dep in dependencies:
        print(f"\t{dep}")
