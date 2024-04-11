use std::fs;
use crate::Color;
use crate::Picture;
use crate::Fsm;

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
    let mut start = Fsm::builder(&Picture::open_pic("./builtin/FStart.png"))
        .add_input(Color::from(0, 148, 255))
        .add_output(Color::from(178, 0, 255))
        .build();

    print!("{:?}", start.identify(p));
}
