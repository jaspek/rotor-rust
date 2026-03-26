// Test 2: Bug triggers when first two bytes of argv[1] encode value 0x4142 ('A','B').
// Written in C* (selfie subset): only uint64_t types, pointer arithmetic.
// Computes val = byte0 * 256 + byte1. Triggers bad exit when val == 16706 (0x4142).
// Compile: selfie -c test2_numeric_overflow.c -m 1
// Rotor:   rotor test2_numeric_overflow.m --symbolic-argv --symbolic-argc 1 --max-arglen 8

uint64_t main(uint64_t argc, uint64_t* argv) {
    uint64_t* arg1;
    uint64_t word;
    uint64_t byte0;
    uint64_t byte1;
    uint64_t val;

    if (argc > 1) {
        arg1 = (uint64_t*) *(argv + 1);
        word = *arg1;
        // Extract first two bytes via modulo
        byte0 = word - (word / 256) * 256;
        byte1 = (word / 256) - ((word / 256) / 256) * 256;

        val = byte0 * 256 + byte1;

        // Bad exit when val == 16706 (byte0 == 'A' == 65, byte1 == 'B' == 66)
        if (val == 16706)
            return 1;
    }

    return 0;
}
