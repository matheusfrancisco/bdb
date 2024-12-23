#!/usr/bin/env python3

for length in range(1, 20):
    print(chr(ord('a') + length % 26) * (20 * length))
    print(chr(ord('a') + length % 26) * (20 * length))
