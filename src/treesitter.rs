use std::collections::HashSet;
use tree_sitter::Language as TSLanguage;
use tree_sitter::Node as TSNode;
use tree_sitter::Parser as TSParser;
use tree_sitter::Query as TSQuery;
use tree_sitter::QueryCapture as TSQueryCapture;
use tree_sitter::Tree as TSTree;

extern "C" {
    fn tree_sitter_lua() -> TSLanguage;
}

/// Parser trait is an abstraction over the source code and the tree-sitter internal structures
/// that represent AST.
pub trait Parser {
    /// Returns name of the language.
    fn name(&self) -> String;

    /// Returns a tree-sitter Language.
    fn language(&self) -> &TSLanguage;

    /// Returns a tree-sitter tree.
    fn tree(&self) -> &TSTree;

    /// Returns a query that extracts imports for the given language.
    fn imports_query(&self) -> String;

    /// Removes `nodes` from the given `source_code`. Returns the source code after this
    /// transformation.
    fn remove_nodes<'a>(&self, source_code: &str, nodes: &[TSNode<'a>]) -> Result<String, String>;

    /// Performs a tree-sitter query for previously set source code and collects matched
    /// tree-sitter nodes.
    ///
    /// Arguments:
    /// * `source_code` - Source code of the program.
    /// * `query_text` - String representation of the tree-sitter query.
    /// * `filter` - A lambda function that filters collected captures. This is required to
    ///              work around the bugs in some tree-sitter parsers.
    fn get_matches(
        &self,
        source_code: &str,
        query_text: String,
        filter: Option<fn(&&TSQueryCapture) -> bool>,
    ) -> Vec<TSNode<'_>> {
        let root_node = self.tree().root_node();
        let query = TSQuery::new(*self.language(), &query_text).unwrap();
        let mut query_cursor = tree_sitter::QueryCursor::new();
        let matches = query_cursor.matches(&query, root_node, source_code.as_bytes());
        let filter = filter.unwrap_or(|_| true);
        matches.fold(Vec::new(), |mut acc, m| {
            acc.extend(
                m.captures
                    .iter()
                    .filter(|c| !c.node.has_error())
                    .filter(filter)
                    .map(|c| c.node),
            );
            acc.clone()
        })
    }
}

/// Returns source code of the given node. For debugging purposes.
#[allow(dead_code)]
pub fn node_source(source: &str, node: &TSNode<'_>) -> String {
    source[node.start_byte()..node.end_byte()].to_string()
}

pub struct Lua {
    language: TSLanguage,
    tree: TSTree,
}

impl Lua {
    pub fn new(source_code: &str) -> Result<Lua, String> {
        let language = unsafe { tree_sitter_lua() };
        let mut parser = TSParser::new();
        // TODO: Set timeout for the parsing
        parser.set_language(language).unwrap();
        let tree = match parser.parse(source_code, None) {
            Some(tree) => tree,
            None => return Err("Cannot parse the given source".to_string()),
        };
        Ok(Lua { language, tree })
    }
}

impl Parser for Lua {
    fn name(&self) -> String {
        "Lua".to_string()
    }

    fn language(&self) -> &TSLanguage {
        &self.language
    }

    fn tree(&self) -> &TSTree {
        &self.tree
    }

    fn imports_query(&self) -> String {
        "((function_call
            prefix: ((identifier) @p (#match? @p \"require\"))
            args: (function_arguments) @args) @func_call)"
            .to_string()
    }

    fn remove_nodes<'a>(&self, source_code: &str, nodes: &[TSNode<'a>]) -> Result<String, String> {
        let nodes_to_remove: HashSet<TSNode<'a>> = HashSet::from_iter(nodes.iter().cloned());
        let mut removed_ranges = Vec::new();
        for node in self.tree.root_node().children(&mut self.tree.walk()) {
            if !nodes_to_remove.contains(&node) {
                continue;
            }
            // TODO: We could use tree-sitter edits to implement the incremental parsing and make
            // it faster. But this will require changes in the parser to make it stateful.
            // edits.push(TSInputEdit {
            //     start_byte: node.start_byte(),
            //     old_end_byte: node.end_byte(),
            //     new_end_byte: node.start_byte(),
            //     start_position: node.start_position(),
            //     old_end_position: node.end_position(),
            //     new_end_position: node.start_position(),
            // });
            removed_ranges.push((node.start_byte(), node.end_byte()));
        }

        // Sort removed ranges in descending order, because we will remove symbols from the end to
        // don't break the previous positions.
        removed_ranges.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        // Remove removed ranges from the program code.
        let mut source: Vec<u8> = source_code.as_bytes().to_vec();
        let source = match std::str::from_utf8(
            removed_ranges
                .iter()
                .fold(&mut source, |source_bytes, &range| {
                    source_bytes.drain(range.0..range.1);
                    source_bytes
                })
                .as_slice(),
        ) {
            Ok(v) => v.to_string(),
            Err(err) => {
                return Err(format!(
                    "Invalid UTF-8 sequence in the source code: {}",
                    err
                ))
            }
        };

        Ok(source)
    }
}
