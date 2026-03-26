// Test 1: Bug triggers when argv[1] == "CRASH"
// This is the canonical example from the proposal.
// The solver should find argv[1] = "CRASH" as the triggering input.

int main(int argc, char* argv[]) {
    if (argc < 2) return 0;

    char* s = argv[1];

    if (s[0] == 'C')
        if (s[1] == 'R')
            if (s[2] == 'A')
                if (s[3] == 'S')
                    if (s[4] == 'H')
                        return 1; // bad exit code

    return 0;
}
