use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use petgraph::Graph;
use petgraph::dot::{Dot, Config};
use petgraph::graph::NodeIndex;
use std::sync::Mutex;

#[pyclass]
pub struct MemGraph {
    graph: Mutex<Graph<&'static str, ()>>,
}

#[pymethods]
impl MemGraph {
    #[new]
    pub fn new(file_path: &str) -> Self {
        let mut graph = Graph::new();
        let _a = graph.add_node("A");
        let _b = graph.add_node("B");
        let _c = graph.add_node("C");
        graph.add_edge(_a, _b, ());
        graph.add_edge(_b, _c, ());
        graph.add_edge(_c, _a, ());

        MemGraph {
            graph: Mutex::new(graph),
        }
    }

    pub fn output_dot(&self) -> PyResult<String> {
        let graph = self.graph.lock().unwrap();
        let dot_format = Dot::with_config(&*graph, &[Config::EdgeNoLabel]);
        Ok(format!("{:?}", dot_format))
    }
}

#[pymodule]
fn mem_to_graph(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MemGraph>()?;
    Ok(())
}
