// Test 4: Bug requires TWO specific arguments: argv[1][0]=='X' AND argv[2][0]=='Y'.
// Tests multi-argument symbolic reasoning (--symbolic-argc 2).
// CANNOT be found via symbolic stdin alone (program never reads stdin).

int main(int argc, char* argv[]) {
    if (argc < 3) return 0;

    char a = argv[1][0];
    char b = argv[2][0];

    if (a == 'X')
        if (b == 'Y')
            return 1; // bad exit code

    return 0;
}
