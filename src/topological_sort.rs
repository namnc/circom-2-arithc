use crate::compiler::CircuitError;

pub fn topological_sort(
    len: usize,
    get_deps: &dyn Fn(usize) -> Vec<usize>,
) -> Result<Vec<usize>, CircuitError> {
    let mut sorted = Vec::with_capacity(len);
    let mut visiting = vec![false; len];
    let mut visited = vec![false; len];

    for i in 0..len {
        topological_sort_visit(i, &mut visiting, &mut visited, get_deps, &mut sorted)?;
    }

    assert!(
        sorted.len() == len,
        "Topological sort did not return all elements"
    );

    Ok(sorted)
}

fn topological_sort_visit(
    i: usize,
    visiting: &mut [bool],
    visited: &mut [bool],
    get_deps: &dyn Fn(usize) -> Vec<usize>,
    sorted: &mut Vec<usize>,
) -> Result<(), CircuitError> {
    if visited[i] {
        return Ok(());
    }

    if visiting[i] {
        return Err(CircuitError::CyclicDependency {
            message: format!("detected at i={}", i),
        });
    }

    visiting[i] = true;

    for j in get_deps(i) {
        topological_sort_visit(j, visiting, visited, get_deps, sorted)?;
    }

    sorted.push(i);
    visited[i] = true;

    Ok(())
}
