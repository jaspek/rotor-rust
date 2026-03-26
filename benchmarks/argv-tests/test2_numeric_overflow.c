// Test 2: Bug triggers when argv[1] encodes a specific 2-byte value.
// Manual two-character "number": treats argv[1][0] and argv[1][1] as a 16-bit value.
// Triggers division by zero when the computed value equals 256.
// CANNOT be found via symbolic stdin alone (program never reads stdin).

int main(int argc, char* argv[]) {
    if (argc < 2) return 0;

    char* s = argv[1];
    int val = ((int)s[0]) * 256 + (int)s[1];

    // Division by zero when val == 256 (s[0] == 1, s[1] == 0... but s[1]==0 is null terminator)
    // Actually: val - 0x4142 == 0 when s[0]=='A' and s[1]=='B'
    int divisor = val - 0x4142; // 'A'*256 + 'B'

    if (divisor == 0) {
        // Division by zero: bad state
        int x = 42 / divisor;
        return x;
    }

    return 0;
}
