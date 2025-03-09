import string
import itertools

print("pub const TABLE: [u8; 64] = [")

for c in string.ascii_uppercase + string.ascii_lowercase + string.digits + "+" + "/":
    print(f"{ord(c)},", end="")

print("];")

print("""pub const REVERSE_TABLE: [u32; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 0, 0, 0, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
    22, 23, 24, 25, 0, 0, 0, 0, 0, 0, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];""")

print("pub const REVERSE_PAIR_TABLE: [u16; 65535] = [", end="")

indexes = {k: v for k, v in zip(string.ascii_uppercase + string.ascii_lowercase + string.digits + "+" + "/", range(64))}
pairs = list(itertools.product(string.ascii_uppercase + string.ascii_lowercase + string.digits + "+" + "/", repeat=2))
pairs_indexes = {ord(pair[0]) | ord(pair[1]) << 8: indexes[pair[0]] << 6 | indexes[pair[1]] for pair in pairs}

for i in range(65535):
    print(f"{pairs_indexes.get(i, 0)},", end="")

print("];")

pairs_indexes = {indexes[pair[0]] << 6 | indexes[pair[1]]: ord(pair[0]) | ord(pair[1]) << 8 for pair in pairs}

print("pub const PAIR_TABLE: [u32; 4096] = [")

for i in range(4096):
    print(f"{pairs_indexes.get(i, 0)},", end="")

print("];")
