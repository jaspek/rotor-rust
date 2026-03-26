// Test 3: Bug triggers when first byte of argv[1] is non-zero and second is zero.
// Written in C* (selfie subset): only uint64_t types, pointer arithmetic.
// This is a simplified length check: string of exactly length 1 triggers the bug.
// Compile: selfie -c test3_length_dependent.c -m 1
// Rotor:   rotor test3_length_dependent.m --symbolic-argv --symbolic-argc 1 --max-arglen 8

uint64_t main(uint64_t argc, uint64_t* argv) {
    uint64_t* arg1;
    uint64_t word;
    uint64_t byte0;
    uint64_t byte1;

    if (argc > 1) {
        arg1 = (uint64_t*) *(argv + 1);
        word = *arg1;
        byte0 = word - (word / 256) * 256;
        byte1 = (word / 256) - ((word / 256) / 256) * 256;

        // String of length exactly 1: first byte != 0, second byte == 0
        if (byte0 != 0)
            if (byte1 == 0)
                return 1;
    }

    return 0;
}
