mod common;

use crate::common::Test;

#[test]
fn lua_top_1() {
    Test::new()
        .source(
            "
function foo()  assert(false) end
function bar()  return false  end
function baz()  assert(false) end
function main() foo() end
",
        )
        .script("! grep -q -E \"assert\\(false\\)\" $1")
        .passes("top")
        .check_reduced("function foo()  assert(false) end");
}
