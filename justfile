build-cov:
    RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="hf-%p-%m.profraw" cargo build

test-cov:
    RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="hf-%p-%m.profraw" cargo nextest run

coverage-report:
    rm -rf target/coverage
    mkdir -p target/coverage
    grcov . \
        --binary-path ./target/debug/ \
        --source-dir . \
        --output-type html \
        --branch \
        --ignore-not-existing \
        --ignore "**/tests/*" \
        --ignore "**/build.rs" \
        --excl-line "// LCOV_EXCL_LINE" \
        --excl-start "// LCOV_EXCL_START" \
        --excl-stop "// LCOV_EXCL_STOP" \
        --output-path target/coverage

serve-coverage:
    miniserve target/coverage

clean-cov:
    find . -type f -name '*.profraw' -delete

cov:
    just clean-cov
    just build-cov
    just test-cov
    just coverage-report
    just serve-coverage
