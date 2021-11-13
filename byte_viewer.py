from sys import argv
from os.path import abspath

if __name__ == "__main__":
    args = argv[1:]
    rel = abspath(args[0])
    with open(rel, "rb") as f:
        arr = f.read()
        for v in arr:
            print(v, end=" ")
        print()