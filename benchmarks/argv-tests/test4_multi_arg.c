// Test 4: Bug requires TWO specific arguments: argv[1][0] == 88 ('X') AND argv[2][0] == 89 ('Y').
// Written in C* (selfie subset): only uint64_t types, pointer arithmetic.
// Tests multi-argument symbolic reasoning (requires --symbolic-argc 2).
// Compile: selfie -c test4_multi_arg.c -m 1
// Rotor:   rotor test4_multi_arg.m --symbolic-argv --symbolic-argc 2 --max-arglen 8

uint64_t main(uint64_t argc, uint64_t* argv) {
    uint64_t* arg1;
    uint64_t* arg2;
    uint64_t word1;
    uint64_t word2;
    uint64_t byte1;
    uint64_t byte2;

    if (argc > 2) {
        arg1 = (uint64_t*) *(argv + 1);
        arg2 = (uint64_t*) *(argv + 2);
        word1 = *arg1;
        word2 = *arg2;
        byte1 = word1 - (word1 / 256) * 256;
        byte2 = word2 - (word2 / 256) * 256;

        if (byte1 == 88)
            if (byte2 == 89)
                return 1;
    }

    return 0;
}
