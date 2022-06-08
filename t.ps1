#clear
$now = Date
echo "==================================== new cargo test ==========================================="
echo "                                       $now"
echo "==================================== new cargo test ==========================================="
#cargo test --features extended,html -- --nocapture
cargo test --features extended -j 1 $args[0] -- --nocapture