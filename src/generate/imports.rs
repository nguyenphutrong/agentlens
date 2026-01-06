use crate::analyze::FileGraph;

pub fn generate_imports(graph: &FileGraph) -> String {
    let mut output = String::new();

    output.push_str("# imports.md\n\n");
    output.push_str("File dependency graph showing imports and importers.\n\n");

    if graph.imports.is_empty() {
        output.push_str("*No import relationships detected.*\n");
        return output;
    }

    let mut files: Vec<_> = graph.imports.keys().collect();
    files.sort();

    for file in files {
        let imports = graph.imports.get(file).cloned().unwrap_or_default();
        let importers = graph.importers.get(file).cloned().unwrap_or_default();

        if imports.is_empty() && importers.is_empty() {
            continue;
        }

        output.push_str(&format!("## `{}`\n\n", file));

        if !imports.is_empty() {
            let mut sorted_imports = imports.clone();
            sorted_imports.sort();
            output.push_str("**Imports:** ");
            output.push_str(&sorted_imports.join(", "));
            output.push_str("\n\n");
        } else {
            output.push_str("**Imports:** (none)\n\n");
        }

        if !importers.is_empty() {
            let mut sorted_importers = importers.clone();
            sorted_importers.sort();
            output.push_str("**Imported by:** ");
            output.push_str(&sorted_importers.join(", "));
            output.push_str("\n\n");
        } else {
            output.push_str("**Imported by:** (none - entry point)\n\n");
        }

        output.push_str("---\n\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = FileGraph::new();
        let result = generate_imports(&graph);
        assert!(result.contains("No import relationships detected"));
    }

    #[test]
    fn test_with_imports() {
        let mut graph = FileGraph::new();
        graph.add_file("main.rs", vec!["lib".to_string(), "utils".to_string()]);
        graph.add_file("lib.rs", vec!["types".to_string()]);

        let result = generate_imports(&graph);
        assert!(result.contains("main.rs"));
        assert!(result.contains("lib, utils"));
    }
}
