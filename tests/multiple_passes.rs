mod common;

use crate::common::Test;

#[test]
fn multiple_passes_1() {
    Test::new()
        .source(
            "
require(\"mod1\")
function main() print(\"test\") end
function test() assert(false) end
",
        )
        .script(
            "! grep -q -E \"assert\\(false\\)\" $1 && ! grep -q -E \'require\\(\"mod1\"\\)\' $1",
        )
        .passes("top;imports")
        .check_reduced("require(\"mod1\")");
}
