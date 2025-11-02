SCRIPT_DIR=$(dirname "$0")
cd "$SCRIPT_DIR" || exit
rm -r output
cp -r input output
cargo run -- --fix -vvv --skip-tags not_selected -p pyproject.toml
diff -w -B expected_output output
if diff -w -B expected_output output; then
    echo "Expected output matches actual output"
else
    echo "Expected output does not match actual output"
fi
