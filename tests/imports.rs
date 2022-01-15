mod common;

use crate::common::Test;

#[test]
fn lua_requires_1() {
    Test::new()
        .source(
            "require(\"mod1\")
require(\"mod2\")
require(\"mod3\")
require(\"mod4\")
",
        )
        .script("! grep -q -E \"require\\(\\\"mod2\\\"\\)\" $1")
        .passes("imports")
        .check_reduced("require(\"mod2\")");
}
