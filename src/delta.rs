//! The delta module contains implementation of the common delta debugging algorithms.

use crate::passes::{Pass, TestOutcome};
use tree_sitter::Node as TSNode;

/// Remove all nodes occurred in the `complement`, because they doesn't cause a fail.
fn remove_complement<'a>(seq: &mut Vec<TSNode<'a>>, complement: &Vec<TSNode<'a>>) {
    let new_seq: Vec<TSNode> = seq.iter().fold(vec![], |mut acc, node| {
        if !complement.contains(&node) {
            acc.push(*node);
        }
        acc
    });
    let new_seq: &[TSNode<'a>] = &new_seq;
    seq.drain(new_seq.len()..);
    seq.clone_from_slice(new_seq);
}

/// Reduces the current sequence, using the outcome of the `test` function.
/// It returns the nodes that causes the failure and the source code of the minimal reproducible
/// example.
///
/// This function implements the Minimizing Delta Debugging algorithm described in
/// [Zeller et al, 2002](https://doi.org/10.1109/32.988498).
pub fn ddmin<'a>(
    seq: &Vec<TSNode<'a>>,
    pass: &impl Pass<'a>,
) -> Result<(Vec<TSNode<'a>>, String), String> {
    let mut source_code = pass.original_source().clone();
    match pass.test_source(&source_code) {
        Ok((TestOutcome::PASS, _)) => {
            return Err("`test` succeeds for the given AST root".to_string())
        }
        Ok(_) => (),
        Err(err) => return Err(err),
    };

    let mut granularity = 2;
    let mut seq = seq.clone();
    while seq.len() >= 2 {
        let mut start: usize = 0;
        let subset_length: usize = (seq.len() / 2) as usize;
        let mut some_complement_is_failing = false;
        while start < seq.len() {
            // A complement is a sequence of nodes that will be removed during the test.
            let complement = [&seq[..start], &seq[start + subset_length..]].concat();
            log::debug!(
                "Testing seq: {:?}",
                complement
                    .iter()
                    .map(|n| { crate::treesitter::node_source(&pass.original_source(), n) })
                    .collect::<Vec<String>>()
            );
            if let Ok((TestOutcome::FAIL, new_source)) = pass.test_nodes(&source_code, &complement)
            {
                remove_complement(&mut seq, &complement);
                source_code = new_source;
                log::debug!(
                    "Reduced seq: {:?} New source: {}",
                    seq.iter()
                        .map(|n| { crate::treesitter::node_source(&pass.original_source(), n) })
                        .collect::<Vec<String>>(),
                    source_code
                );
                granularity = std::cmp::max(granularity - 1, 2);
                some_complement_is_failing = true;
                break;
            }
            start += subset_length;
        }
        if !some_complement_is_failing {
            if granularity == seq.len() {
                break;
            }
            granularity = std::cmp::min(granularity * 2, seq.len());
        }
    }
    Ok((seq.to_vec(), source_code))
}
