#!/bin/bash
cd /selfie
make selfie 2>/dev/null

echo "=== Compiling argv tests ==="
for f in /benchmarks/argv-tests/test*.c; do
    name=$(basename "$f" .c)
    echo -n "$name: "
    if ./selfie -c "$f" -o "/benchmarks/argv-tests/${name}.m" 2>&1 | grep -q "bytes generated"; then
        echo "OK"
    else
        echo "FAILED"
        ./selfie -c "$f" 2>&1 | grep -i "error\|syntax\|unexpected" | head -3
    fi
done

echo ""
echo "=== Generated binaries ==="
ls -la /benchmarks/argv-tests/*.m 2>/dev/null || echo "No .m files generated"
