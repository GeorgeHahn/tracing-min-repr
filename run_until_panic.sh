#!/bin/bash

# Script to build once then run the binary 1000 times and tally runs with and without panics

echo "Building project first..."
cargo_build_output=$(cargo build --release 2>&1)
build_exit_code=$?

if [ $build_exit_code -ne 0 ]; then
    echo "❌ BUILD FAILED!"
    echo "Build output:"
    echo "----------------------------------------"
    echo "$cargo_build_output"
    echo "----------------------------------------"
    exit $build_exit_code
fi

# Get the project name from Cargo.toml
project_name=$(grep '^name = ' Cargo.toml | sed 's/name = "\(.*\)"/\1/')
binary_path="target/release/$project_name"

if [ ! -f "$binary_path" ]; then
    echo "❌ Binary not found at $binary_path"
    echo "Available files in target/release/:"
    ls -la target/release/ 2>/dev/null || echo "target/release/ directory not found"
    exit 1
fi

echo "✅ Build successful! Binary at: $binary_path"
echo "Running binary 1000 times and tallying results..."
echo "Press Ctrl+C to stop manually"
echo ""

panic_count=0
success_count=0
other_failure_count=0
panic_example=""
success_example=""

for i in {1..1000}; do
    # Run the binary directly and capture both stdout and stderr
    output=$(./"$binary_path" 2>&1)
    exit_code=$?

        # Check for panic indicators in the output
    if echo "$output" | grep -q -E "(panicked at|thread.*panicked|stack backtrace:)"; then
        panic_count=$((panic_count + 1))

        # Capture first panic example for display at end
        if [ -z "$panic_example" ]; then
            panic_example="$output"
        fi

        # Optionally save panic output to a file for later analysis
        echo "=== Run #$i - PANIC ===" >> panic_outputs.log
        echo "$output" >> panic_outputs.log
        echo "" >> panic_outputs.log
    elif [ $exit_code -ne 0 ]; then
        other_failure_count=$((other_failure_count + 1))

        # Optionally save failure output to a file for later analysis
        echo "=== Run #$i - FAILED (exit code: $exit_code) ===" >> failure_outputs.log
        echo "$output" >> failure_outputs.log
        echo "" >> failure_outputs.log
    else
        success_count=$((success_count + 1))

        # Capture first success example for display at end
        if [ -z "$success_example" ]; then
            success_example="$output"
        fi
    fi

    # Show progress every 100 runs
    if [ $((i % 100)) -eq 0 ]; then
        echo ""
        echo "Progress: $i/1000 runs completed"
        echo "  ✅ Successes: $success_count"
        echo "  ❌ Panics: $panic_count"
        echo "  ⚠️  Other failures: $other_failure_count"
        echo ""
    fi
done

echo ""
echo "🎯 FINAL RESULTS after 1000 runs:"
echo "  ✅ Successful runs: $success_count"
echo "  ❌ Panic runs: $panic_count"
echo "  ⚠️  Other failure runs: $other_failure_count"
echo ""
echo "Statistics:"
echo "  Success rate: $(echo "scale=2; $success_count * 100 / 1000" | bc)%"
echo "  Panic rate: $(echo "scale=2; $panic_count * 100 / 1000" | bc)%"
echo "  Other failure rate: $(echo "scale=2; $other_failure_count * 100 / 1000" | bc)%"

if [ $panic_count -gt 0 ]; then
    echo ""
    echo "Panic outputs saved to: panic_outputs.log"
fi

if [ $other_failure_count -gt 0 ]; then
    echo ""
    echo "Other failure outputs saved to: failure_outputs.log"
fi

echo ""
echo "=========================================="
echo "EXAMPLE OUTPUTS:"
echo "=========================================="

if [ -n "$success_example" ]; then
    echo ""
    echo "📋 SUCCESS EXAMPLE:"
    echo "------------------------------------------"
    echo "$success_example"
    echo "------------------------------------------"
fi

if [ -n "$panic_example" ]; then
    echo ""
    echo "💥 PANIC EXAMPLE:"
    echo "------------------------------------------"
    echo "$panic_example"
    echo "------------------------------------------"
fi