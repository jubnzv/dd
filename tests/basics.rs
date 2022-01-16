mod common;

use crate::common::Test;

#[test]
fn basics_1() {
    Test::new()
        .source("function main() print(\"test\") end")
        .script("! grep -q -E \"assert\\(false\\)\" $1")
        .passes("top")
        .check_not_reduced();
}
