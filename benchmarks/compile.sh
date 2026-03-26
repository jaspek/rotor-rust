#!/bin/bash
set -e

echo "=== Building Selfie toolchain ==="
cd /selfie
make selfie 2>&1

echo ""
echo "=== Building C Rotor ==="
make rotor 2>&1 || echo "WARN: rotor make target may need adjustment"

echo ""
echo "=== Compiling benchmark programs to RISC-U ELF ==="
mkdir -p /output/binaries
mkdir -p /output/btor2-c-rotor

# The 18 symbolic test programs
for src in /selfie/examples/symbolic/*.c; do
    name=$(basename "$src" .c)
    echo "Compiling: $name"

    # Compile to RISC-U ELF using selfie's C* compiler
    ./selfie -c "$src" -o "/output/binaries/${name}.m" 2>&1 || {
        echo "  WARN: Failed to compile $name"
        continue
    }

    echo "  -> /output/binaries/${name}.m"
done

echo ""
echo "=== Generating reference BTOR2 models with C Rotor ==="
for src in /selfie/examples/symbolic/*.c; do
    name=$(basename "$src" .c)
    echo "Generating BTOR2: $name"

    # C rotor writes the BTOR2 file next to the source as *-rotorized.btor2
    ./rotor -m64 -c "$src" - 0 2>&1 || {
        echo "  WARN: C rotor failed for $name"
        continue
    }

    # Copy the generated BTOR2 file
    btor2_file="/selfie/examples/symbolic/${name}-rotorized.btor2"
    if [ -f "$btor2_file" ]; then
        cp "$btor2_file" "/output/btor2-c-rotor/${name}.btor2"
        lines=$(wc -l < "/output/btor2-c-rotor/${name}.btor2")
        echo "  -> ${lines} lines"
    else
        echo "  WARN: No BTOR2 file generated for $name"
    fi
done

echo ""
echo "=== Summary ==="
echo "Binaries:"
ls -la /output/binaries/ 2>/dev/null || echo "  (none)"
echo ""
echo "C Rotor BTOR2 models:"
ls -la /output/btor2-c-rotor/ 2>/dev/null || echo "  (none)"
echo ""
echo "Done!"
