#![allow(dead_code)]
use crate::picture::*;
use crate::point::*;

use std::collections::HashMap;

/// The different transitions between states that are possible
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transition {
    MoveRelative(usize, Point), // Moves relative to a state
    Consume(Color), // Move relative to a point from that state then consuME
    Epsilon, // Change states for free
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorType {
    Input,
    Output,
    Function,
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
    pub colors: HashMap<Color, ColorType>,
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

        let mut colors = HashMap::new();
        colors.insert(func_color, ColorType::Function);

        if head_pos.x == -1 || head_pos.y == -1 {
            panic!("No function color found in FSM definition");
        }

        return FSMBuilder{
            states: vec![State::new()],
            head_pos,
            p,
            colors,
        };
    }

    pub fn identify(&self, p: Picture) -> Option<HashMap<Color, Vec<Color>>> {
        fn recurse(head: Point, p: Picture, f: &FSM, state_index: i32, state_points: Vec<Point>, collect: HashMap<Color, Vec<Color>>) -> Option<HashMap<Color, Vec<Color>>> {
            let cur_state = f.states[state_index as usize].clone();
            // If the State we are in is empty, we are in the end, return all the colors we have
            // collected!
            if cur_state.t.len() == 0 {
                return Some(collect);
            }
            // Loop through the transitions in this state and see if any of them work
            for (destination, transition) in cur_state.t {
                match transition {
                    Transition::MoveRelative(rel_state, direction) => {
                        // We need to change the destination to our head because we are entering
                        // Set it to the relative state's point + direction
                        let new_head = state_points[rel_state] + direction;
                        // Check to make sure still in bounds
                        if !p.in_bounds(new_head) {
                            return None
                        }
                        // clone state_points so nothing gets changed that shouldnt
                        let mut new_points = state_points.clone();
                        new_points[destination] = new_head;
                        // If the recursion gives us a result then perfect return it, if not then
                        // just go to next transition
                        if let Some(result) = recurse(new_head, p.clone(), f, destination as i32, new_points, collect.clone()) {
                            return Some(result);
                        }
                        else {
                            continue;
                        }
                    }
                    Transition::Consume(color) => {
                        let head_color = p.get(head.x, head.y);
                        // We never want to consume white, so if it's white don't waste our time
                        if head_color == WHITE {
                            continue;
                        }
                        let mut new_collect = collect.clone();
                        // We can safely unwrap because we know we have inserteed every used color
                        let color_list = new_collect.get_mut(&color).unwrap();

                        color_list.push(head_color);
                        // Sets it to white because we have collected it
                        let mut new_picture = p.clone();
                        new_picture.set(head.x, head.y, WHITE);

                        // If the recursion gives us a result then perfect return it, if not then
                        // just go to next transition
                        if let Some(result) = recurse(head, new_picture, f, destination as i32, state_points.clone(), new_collect) {
                            return Some(result);
                        }
                        else {
                            continue;
                        }
                    }
                    Transition::Epsilon => {}
                }
            }
            // If none of those transitiions work 
            return None;
        }

        let mut collect: HashMap<Color, Vec<Color>> = HashMap::new();
        // Create collection bins for the consuming of colors
        let _ = self.colors.keys().for_each(|k| {
            collect.insert(k.clone(), vec![]);
        });

        // Get function color
        let func_color = self.colors.iter().find_map(|(key, &value)| if value == ColorType::Function {Some(key)} else {None}  ).unwrap();
        
        // Find toppest leftest function color
        let mut head_pos = Point::from(-1, -1);
        'outer: for j in 0..p.height {
            for i in 0..p.width {
                if p.get(i, j) == *func_color {
                    head_pos = Point::from(i,j);
                    break 'outer;
                }
            }
        }
        // If there is no function color found it's automatically invalid
        if head_pos.x == -1 || head_pos.y == -1 {
            return None;
        }


        // The index into the current state
        let mut state_index = 0;

        // The entry point for each state
        let mut state_points = vec![Point::from(0, 0); self.states.len()];
        // Make sure to put in where you're entering!
        state_points[0] = head_pos;
        
        // The transition index for each state's list of transitions
        let mut transition_index = vec![0 as usize; self.states.len()];

        return recurse(head_pos, p.clone(), &self, state_index.clone(), state_points.clone(), collect.clone());
    }

    pub fn print(&self) {
        for i in 0..self.states.len() {
            println!("{}: {:?}", i, self.states[i]);
        }
    }
}

pub struct FSMBuilder {
    pub states: Vec<State>,
    pub colors: HashMap<Color, ColorType>,
    pub head_pos: Point,
    pub p: Picture,
}

impl FSMBuilder {
    /// Consumes the color underneath
    /// Adds another empty state afterwards
    fn consume(&mut self, c: Color) {
        let len = self.states.len();
        let color = self.color(c).unwrap().0.clone();
        self.states[len - 1].t.push((len, Transition::Consume(color)));
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
        self.colors.insert(c, ColorType::Input);
    }

    /// Adds output color to look for
    /// Internally all colors have a usize that identifies it
    pub fn add_output(&mut self, c: Color) {
        self.colors.insert(c, ColorType::Output);
    }
    
    /// Identifies if a selected color is significant to the relative finite state machine
    fn color(&self, c: Color) -> Option<(&Color, &ColorType)> {
        return self.colors.get_key_value(&c);
    }

    /// Recursively build a FSM from the FSM Builder
    pub fn build(&mut self) -> FSM {
        self.recurse();
        FSM {
            states: self.states.clone(),
            colors: self.colors.clone(),
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


#[cfg(test)]
mod tests {
    use crate::picture;
    use super::*;
    use std::fs;
    
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
    #[test]
    /// Checks to make sure all the definitions without loops identify themselves
    fn fsm_identifies_self() {
        let paths = fs::read_dir("./tests/definitions").unwrap();
        for path in paths {
            let p = picture::Picture::open_pic(path.unwrap().path().to_str().unwrap());
            let mut fsm_builder = FSM::builder(p.clone());
            let fsm = fsm_builder.build();
            assert!(fsm.identify(p.clone()).is_some());
        }
    }
}
