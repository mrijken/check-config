SCRIPT_DIR=$(dirname "$BASH_SOURCE")
cd $SCRIPT_DIR
rm -r output
cp -r input output
cargo run -- --fix  -vv
diff -w -B expected_output output
if [ $? -eq 0 ]; then
    echo "Expected output matches actual output"
else
    echo "Expected output does not match actual output"
fi
