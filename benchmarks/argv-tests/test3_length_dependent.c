// Test 3: Bug triggers based on the LENGTH of argv[1].
// Counts characters until null terminator. Crashes if length == 7.
// This tests that the solver can reason about string length via symbolic bytes.

int main(int argc, char* argv[]) {
    if (argc < 2) return 0;

    char* s = argv[1];
    int len = 0;

    while (s[len] != 0) {
        len = len + 1;
        if (len > 8) return 0; // bound the loop for model checking
    }

    if (len == 7)
        return 1; // bad exit code: string of exactly length 7

    return 0;
}
