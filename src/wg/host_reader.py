import os
import struct
import sys

output = sys.stdout.buffer
output.write(b"WGCONF1")
with os.scandir(sys.argv[1]) as entries:
    for entry in entries:
        if os.path.splitext(entry.name)[1] != ".conf":
            continue
        try:
            name = entry.name.encode("utf-8")
        except UnicodeEncodeError:
            continue
        with open(entry.path, "rb") as config:
            contents = config.read()
        for value in (name, contents):
            output.write(struct.pack("<Q", len(value)))
            output.write(value)
output.write(struct.pack("<Q", 0))
