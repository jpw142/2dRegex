use crate::Color;
use crate::Picture;

/// List of built in operations
/// For all Binary Operations structure is:
/// lhs, rhs, result
pub enum Operation {
    Assign(Color, Vec<Color>),
    Add(Color, Color, Color),
    Return(Vec<Color>),
}

struct Tree {
    branches: Vec<Node>,
}

struct Node {
    op: Operation,
    next: Box<Node>,
}

pub fn tokenize(p: &Picture) {
    
}
