// Test 5: Bug triggers when sum of first two bytes of argv[1] equals 200.
// Written in C* (selfie subset): only uint64_t types, pointer arithmetic.
// E.g. byte0 = 100, byte1 = 100 -> sum = 200 -> bad exit.
// Compile: selfie -c test5_checksum.c -m 1
// Rotor:   rotor test5_checksum.m --symbolic-argv --symbolic-argc 1 --max-arglen 8

uint64_t main(uint64_t argc, uint64_t* argv) {
    uint64_t* arg1;
    uint64_t word;
    uint64_t byte0;
    uint64_t byte1;
    uint64_t sum;

    if (argc > 1) {
        arg1 = (uint64_t*) *(argv + 1);
        word = *arg1;
        byte0 = word - (word / 256) * 256;
        byte1 = (word / 256) - ((word / 256) / 256) * 256;

        sum = byte0 + byte1;

        if (sum == 200)
            return 1;
    }

    return 0;
}
