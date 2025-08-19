pub struct Graph<T> {
    nodes: Vec<Node<T>>,
    edges: Vec<Edge>,
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
        self.nodes.push(Node {
            data,
            first_edge: None,
        });
        index
    }

    pub fn get_node(&self, index: NodeIndex) -> Option<&Node<T>> {
        self.nodes.get(index)
    }

    pub fn get_nodes(&self) -> impl Iterator<Item = (NodeIndex, &Node<T>)> {
        self.nodes.iter().enumerate()
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, direction: Direction) {
        let edge_index = self.edges.len();
        let node_data = &mut self.nodes[source];

        self.edges.push(Edge {
            target,
            direction,
            next_edge: node_data.first_edge,
        });

        node_data.first_edge = Some(edge_index);
    }

    pub fn edges(&self, source: NodeIndex, direction: Direction) -> Edges<'_, T> {
        let first_edge = self.nodes[source].first_edge;
        Edges {
            graph: self,
            current_edge_index: first_edge,
            direction,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Node<T> {
    pub data: T,
    first_edge: Option<EdgeIndex>,
}

pub type NodeIndex = usize;

struct Edge {
    target: NodeIndex,
    direction: Direction,
    next_edge: Option<EdgeIndex>,
}

type EdgeIndex = usize;

#[derive(Eq, PartialEq)]
pub enum Direction {
    Outgoing,
    Incoming,
}

pub struct Edges<'graph, T> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
    direction: Direction,
}

impl<'graph, T> Iterator for Edges<'graph, T> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let mut edge = &self.graph.edges[edge_num];

                while self.direction != edge.direction {
                    if let Some(next_edge_index) = edge.next_edge {
                        edge = &self.graph.edges[next_edge_index];
                    } else {
                        self.current_edge_index = None;
                        return None;
                    }
                }

                self.current_edge_index = edge.next_edge;
                Some(edge.target)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut graph: Graph<&str> = Graph::new();

        let index = graph.add_node("a");
        assert_eq!(graph.nodes.len(), 1);

        let node = graph.get_node(index).unwrap();
        assert_eq!(node.data, "a");
    }

    #[test]
    fn test_add_edge() {
        let mut graph: Graph<&str> = Graph::new();

        let a_index = graph.add_node("a");
        let b_index = graph.add_node("b");

        graph.add_edge(a_index, b_index, Direction::Outgoing);

        let edges = graph
            .edges(a_index, Direction::Outgoing)
            .collect::<Vec<_>>();

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], b_index);

        graph.add_edge(b_index, a_index, Direction::Incoming);

        let edges = graph
            .edges(b_index, Direction::Incoming)
            .collect::<Vec<_>>();

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], a_index);
    }
}
