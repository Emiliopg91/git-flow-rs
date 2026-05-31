import os
import re
from pathlib import Path

PROJ_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
CARGO_TOML_PATHS = [PROJ_DIR / "Cargo.toml"]
CARGO_TOML_PATHS.extend(PROJ_DIR.glob("*/Cargo.toml"))

if __name__ == "__main__":
    while True:
        value = input("Enter new version (x.y.z): ")

        parts = value.split(".")

        if len(parts) == 3 and all(p.isdigit() for p in parts):
            break

        print("Invalid format (ej: 1.2.3)")

    version_re = re.compile(r'^(\s*version\s*=\s*")[^"]*(".*)$')
    for file in CARGO_TOML_PATHS:
        with open(file, "r", encoding="utf-8") as f:
            lines = f.readlines()

        for i, line in enumerate(lines):
            if version_re.match(line):
                lines[i] = version_re.sub(rf"\g<1>{value}\2", line)
                break

        with open(file, "w", encoding="utf-8") as f:
            f.writelines(lines)
