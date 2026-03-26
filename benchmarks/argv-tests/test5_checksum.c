// Test 5: Bug triggers when a simple checksum of argv[1] equals a magic value.
// Computes: sum = argv[1][0] + argv[1][1] + argv[1][2] + argv[1][3]
// Crashes when sum == 400 (e.g. 'd'(100) + 'd'(100) + 'd'(100) + 'd'(100)).
// This tests that the solver can reason about arithmetic over symbolic bytes.
// CANNOT be found via symbolic stdin alone (program never reads stdin).

int main(int argc, char* argv[]) {
    if (argc < 2) return 0;

    char* s = argv[1];

    // Need at least 4 characters
    if (s[0] == 0) return 0;
    if (s[1] == 0) return 0;
    if (s[2] == 0) return 0;
    if (s[3] == 0) return 0;

    int sum = (int)s[0] + (int)s[1] + (int)s[2] + (int)s[3];

    if (sum == 400)
        return 1; // bad exit code

    return 0;
}
