// Test 1: Bug triggers when first byte of argv[1] equals 67 ('C').
// Written in C* (selfie subset): only uint64_t types, pointer arithmetic.
// The solver should find argv[1][0] = 67 as the triggering input.
// Compile: selfie -c test1_crash_string.c -m 1
// Rotor:   rotor test1_crash_string.m --symbolic-argv --symbolic-argc 1 --max-arglen 8

uint64_t main(uint64_t argc, uint64_t* argv) {
    uint64_t* arg1;
    uint64_t first_word;
    uint64_t first_byte;

    if (argc > 1) {
        arg1 = (uint64_t*) *(argv + 1);
        first_word = *arg1;
        // Extract least significant byte via modulo (C* has no bitwise AND)
        first_byte = first_word - (first_word / 256) * 256;
        if (first_byte == 67)
            return 1;
    }

    return 0;
}
