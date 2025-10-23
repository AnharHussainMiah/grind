#!/usr/bin/env python3
import os
import subprocess
import sys
import shutil

def find_class_in_jars(folder_path, class_name):
    matches = []

    jar_files = [f for f in os.listdir(folder_path) if f.endswith(".jar")]
    total = len(jar_files)

    # Get terminal width or fallback to 80
    try:
        term_width = shutil.get_terminal_size().columns
    except Exception:
        term_width = 80

    for idx, file in enumerate(jar_files, start=1):
        percent = (idx / total) * 100
        line = f"🔎 Searching [{idx}/{total}] ({percent:.1f}%) in: {file} ..."
        # Pad to terminal width to clear leftovers
        print(f"\r\033[K{line}", end="", flush=True)

        jar_path = os.path.join(folder_path, file)
        try:
            result = subprocess.run(
                ["jar", "tf", jar_path],
                capture_output=True, text=True, check=True
            )
        except subprocess.CalledProcessError as e:
            print(f"\n⚠️  Failed to inspect {file}: {e}", file=sys.stderr)
            continue

        if class_name in result.stdout:
            matches.append(jar_path)

    print()  # newline after progress line
    return matches


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python find_class_in_jars.py <folder_path> <class_name>")
        sys.exit(1)

    folder = sys.argv[1]
    classname = sys.argv[2]

    print(f"🔍 Starting search for class '{classname}' in JARs under '{folder}'...\n")
    results = find_class_in_jars(folder, classname)

    if results:
        print(f"✅ Found '{classname}' in {len(results)} JAR file(s):")
        for jar in results:
            print("  -", jar)
    else:
        print(f"❌ No matches found for '{classname}'.")
