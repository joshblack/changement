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

    pub fn get_node(&self, index: NodeIndex) -> Option<&NodeData<T>> {
        self.nodes.get(index)
    }

    pub fn get_nodes(&self) -> impl Iterator<Item = &NodeData<T>> {
        self.nodes.iter()
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

    pub fn edges(&self, source: NodeIndex) -> Edges<'_, T> {
        let first_outgoing_edge = self.nodes[source].first_outgoing_edge;
        Edges {
            graph: self,
            current_edge_index: first_outgoing_edge,
        }
    }
}

pub struct Edges<'graph, T> {
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

pub type NodeIndex = usize;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct NodeData<T> {
    pub data: T,
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

        assert_eq!(graph.get_node(node_a).unwrap().data, "A");
        assert_eq!(graph.get_node(node_b).unwrap().data, "B");
        assert_eq!(graph.get_node(node_c).unwrap().data, "C");

        assert_eq!(
            graph.edges(node_a).collect::<Vec<_>>(),
            vec![node_c, node_b]
        );
        assert_eq!(graph.edges(node_b).collect::<Vec<_>>(), vec![node_c]);
    }
}
