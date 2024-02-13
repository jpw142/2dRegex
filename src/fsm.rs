#![allow(dead_code)]
use crate::picture::*;
use crate::point::*;

use std::fs;


/// The different transitions between states that are possible
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transition {
    MoveRelative(usize, Point), // Moves relative to a state
    Consume(usize), // Move relative to a point from that state then consume
    Epsilon, // Change states for free
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorType {
    Input(usize),
    Output(usize),
    Function,
    None
}

/// An abstract object that contains transitions to other indexes in its parent structure FSM
/// On entering a state the position of the head is stored
#[derive(Debug, Clone)]
pub struct State {
    // Transitions
    pub t: Vec<(usize, Transition)>
}

impl State {
    pub fn new() -> State {
        State{t: vec![]}
    }
}


#[derive(Debug, Clone)]
/// The base struct for creating a finite state machine, storing it, and executing it
pub struct FSM {
    pub states: Vec<State>,
    pub func_color: Color,
    pub input_color: Vec<(Color, usize)>,
    pub output_color: Vec<(Color, usize)>,
}

impl FSM {
    pub fn builder(mut p: Picture) -> FSMBuilder {
        // Get function color
        let result = p.four_corners();
        if result.is_some() {
            p.set(0, 0, WHITE);
            p.set(p.width - 1, 0, WHITE);
            p.set(p.width - 1, p.height - 1, WHITE);
            p.set(0, p.height - 1, WHITE);
        }
        let func_color = result.unwrap_or(BLUE);

        // Find upper left corner of the symbol
        let mut head_pos = Point::from(-1, -1);
        'outer: for j in 0..p.height {
            for i in 0..p.width {
                if p.get(i, j) == func_color {
                    head_pos = Point::from(i,j);
                    break 'outer;
                }
            }
        }

        if head_pos.x == -1 || head_pos.y == -1 {
            panic!("No function color found in FSM definition");
        }

        return FSMBuilder{
            states: vec![State::new()],
            head_pos,
            p,
            func_color,
            input_color: vec![],
            output_color: vec![]
        };
    }

    pub fn doer(self, mut p: Picture) -> FSMDoer {
        todo!()
    }

    pub fn print(&self) {
        for i in 0..self.states.len() {
            println!("{}: {:?}", i, self.states[i]);
        }
    }
}

pub struct FSMBuilder {
    states: Vec<State>,
    p: Picture,
    head_pos: Point,
    func_color: Color,
    input_color: Vec<(Color, usize)>,
    output_color: Vec<(Color, usize)>,

}

impl FSMBuilder {
    /// Consumes the color underneath
    /// Adds another empty state afterwards
    fn consume(&mut self, c: Color) {
        let len = self.states.len();
        let c_type = self.color(c).unwrap();
        self.states[len - 1].t.push((len, Transition::Consume(c_type)));
        self.states.push(State::new());
    }

    /// Move relative to a state
    /// s is optional, if not included it just defaults to the current state
    /// p is relative coordinates to the state
    /// Adds another empty state afterwards
    fn move_rel(&mut self, s: Option<usize>, p: Point) {
        let len = self.states.len();
        self.states[len - 1].t.push((len, Transition::MoveRelative(s.unwrap_or(len - 1), p)));
        self.states.push(State::new());
    }
    
    /// Adds input color to look for
    /// Internally all colors have a usize that identifies it
    pub fn add_input(&mut self, c: Color) {
        self.input_color.push((c, self.input_color.len()));
    }

    /// Adds output color to look for
    /// Internally all colors have a usize that identifies it
    pub fn add_output(&mut self, c: Color) {
        self.output_color.push((c, self.output_color.len()));
    }
    
    /// Identifies if a selected color is significant to the relative finite state machine
    fn color(&self, c: Color) -> Option<usize> {
        if self.func_color == c {
            return Some(0);
        }
        for i in self.input_color.iter() {
            if c == i.0 {
                return Some(i.1)
            }
        }
        for o in self.output_color.iter() {
            if c == o.0 {
                return Some(self.input_color.len() + o.1);
            }
        }
        return None;
    }

    /// Recursively build a FSM from the FSM Builder
    pub fn build(&mut self) -> FSM {
        self.recurse();
        FSM {
            states: self.states.clone(),
            func_color: self.func_color,
            input_color: self.input_color.clone(),
            output_color: self.output_color.clone(),
        }
    }

    fn recurse(&mut self) {
        let head_pos = self.head_pos;
        let cur_state = self.states.len() - 1;
        let color = self.p.get(self.head_pos.x, self.head_pos.y);
        self.consume(color);
        self.p.set(head_pos.x, head_pos.y, WHITE);

        for pos in SURROUNDING {
            let next_position = head_pos + pos;
            if next_position.x < 0 || next_position.x >= self.p.width{
                continue;
            }
            if next_position.y < 0 || next_position.y > self.p.height - 1{
                continue;
            }

            let cur_color = self.p.get(next_position.x, next_position.y);
            // If we don't care about the color of the surrounding pixel go to the next one
            if self.color(cur_color).is_none() {
                continue;
            }

            self.move_rel(Some(cur_state), pos);
            self.head_pos = next_position;
            self.recurse();
        } 
    }
}

/// Pretty self explanatory from the name
/// Does the FSM with all the silly little variables it needs and then returns a result
pub struct FSMDoer {
    // All stares have points associated with them
    states: Vec<(State, Point)>,
    // Index into what state is what
    i: usize,
    p: Picture,
    head_pos: Point,
    collectors: Vec<(Color, usize)>
}

#[cfg(test)]
mod tests {
    use crate::picture;
    use super::*;
    
    #[test]
    /// Checks that all of the fsm definition tests compile into an fsm regardless of correctness
    fn fsm_compiles() {
        // This is where all of the test definitions live
        // It's a lovely place
        // High rent but utilities are low
        let paths = fs::read_dir("./tests/definitions").unwrap();
        for path in paths {
            let p = picture::Picture::open_pic(path.unwrap().path().to_str().unwrap());
            let mut fsm_builder = FSM::builder(p.clone());
            let fsm = fsm_builder.build();
            assert!(fsm.states.len() > 1);
        }
    }
}
