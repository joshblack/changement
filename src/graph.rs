pub struct Graph<T> {
    nodes: Vec<NodeData<T>>,
    edges: Vec<EdgeData>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, data: T) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(NodeData {
            data,
            first_outgoing_edge: None,
        });
        index
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) {
        let edge_index = self.edges.len();
        let node_data = &mut self.nodes[source];

        self.edges.push(EdgeData {
            target,
            next_outgoing_edge: node_data.first_outgoing_edge,
        });

        node_data.first_outgoing_edge = Some(edge_index);
    }

    pub fn edges(&self, source: NodeIndex) -> Edges<T> {
        let first_outgoing_edge = self.nodes[source].first_outgoing_edge;
        Edges {
            graph: self,
            current_edge_index: first_outgoing_edge,
        }
    }
}

struct Edges<'graph, T> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, T> Iterator for Edges<'graph, T> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge.target)
            }
        }
    }
}

type NodeIndex = usize;

struct NodeData<T> {
    data: T,
    first_outgoing_edge: Option<EdgeIndex>,
}

type EdgeIndex = usize;

struct EdgeData {
    target: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph() {
        let mut graph = Graph::new();
        let node_a = graph.add_node("A");
        let node_b = graph.add_node("B");
        let node_c = graph.add_node("C");
        graph.add_edge(node_a, node_b);
        graph.add_edge(node_a, node_c);
        graph.add_edge(node_b, node_c);
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 3);
        assert_eq!(graph.nodes[node_a].first_outgoing_edge.unwrap(), 0);
        assert_eq!(graph.nodes[node_b].first_outgoing_edge.unwrap(), 1);
        assert_eq!(graph.nodes[node_c].first_outgoing_edge, None);
    }
}
