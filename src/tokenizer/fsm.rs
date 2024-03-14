#![allow(dead_code)]
use crate::tokenizer::point::*;
use crate::tokenizer::picture::*;

use std::collections::HashMap;
use std::collections::HashSet;

/// The different transitions between states that are possible
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transition {
    MoveRelative(usize, Point), // Moves relative to x state by p pos
    Consume(Color), // Consume x color at head_pos
    Capture(u8), // Tells identifier to start x capture group 
    EndCapture(u8), // Tells the identifier to stop x capture group
    Epsilon, // Change to destination state for free
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorType {
    Input,
    Output,
    Function,
}

/// An abstract object that contains transitions to other indexes in its parent structure Fsm
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
pub struct Fsm {
    pub states: Vec<State>,
    pub colors: HashMap<Color, ColorType>,
}

impl Fsm {
    pub fn identify(&self, p: Picture) -> Option<HashMap<Color, Vec<Color>>> {
        fn recurse(
            head: Point, 
            p: &Picture, 
            p_consumed: &HashSet<Point>,
            f: &Fsm, 
            state_index: i32, 
            mut state_points: Vec<Point>, 
            collect: HashMap<Color, Vec<Color>>, 
            epsilon: Option<i32>,
            capture_groups: &Vec<(u8, i32)>,
            ended_groups: &HashMap<u8, i32>
            ) -> Option<HashMap<Color, Vec<Color>>> {
            let cur_state = f.states[state_index as usize].clone();
            state_points[state_index as usize] = head;
            // If the State we are in is empty, we are in the end, return all the colors we have
            // collected!
            if cur_state.t.is_empty() {
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
                            continue;
                        }
                        // clone state_points so nothing gets changed that shouldnt
                        let mut new_points = state_points.clone();
                        new_points[destination] = new_head;
                        // If the recursion gives us a result then perfect return it, if not then
                        // just go to next transition
                        if let Some(result) = recurse(new_head, p, p_consumed, f, destination as i32, new_points, collect.clone(), None, capture_groups, ended_groups) {
                            return Some(result);
                        }
                        else {
                            continue;
                        }
                    }
                    Transition::Consume(color) => {
                        let head_color = p.get_point(head);
                        // We never want to consume white, so if it's white don't waste our time
                        if head_color == WHITE || p_consumed.get(&head).is_some(){
                            continue;
                        }
                        let mut new_collect = collect.clone();
                        // We can safely unwrap because we know we have inserteed every used color
                        let color_list = new_collect.get_mut(&color).unwrap();

                        color_list.push(head_color);
                        // Sets it to white because we have collected it
                        let mut new_p_consumed = p_consumed.clone();
                        new_p_consumed.insert(head);

                        // Update capture groups
                        let mut new_capture = vec![];
                        capture_groups.iter().for_each(|(x, c)| {new_capture.push((*x, c + 1))});
                        // If when we consume and the capture group is bigger than an existing
                        // ended capture group then it cant work
                        for (x,c) in new_capture.iter() {
                            if let Some(result) = ended_groups.get(x) {
                                if *c > *result {
                                    continue;
                                }
                            }
                        }
                        // If the recursion gives us a result then perfect return it, if not then
                        // just go to next transition
                        if let Some(result) = recurse(head, p, &new_p_consumed, f, destination as i32, state_points.clone(), new_collect, None, &new_capture, ended_groups) {
                            return Some(result);
                        }
                    }
                    Transition::Epsilon => {
                        // If the last transition was an epsilon
                        if let Some(eps) = epsilon {
                            // And if we are going back to that transition
                            if eps == destination as i32 {
                                // Then just return none to avoid infinite loop
                                continue;
                            }
                        }
                        if let Some(result) = recurse(head, p, p_consumed, f, destination as i32, state_points.clone(), collect.clone(), Some(state_index), capture_groups, ended_groups) {
                            return Some(result);
                        }
                    }
                    Transition::Capture(g) => {
                        let mut new_capture = capture_groups.clone();
                        new_capture.push((g, 0));
                        if let Some(result) = recurse(head, p, p_consumed, f, destination as i32, state_points.clone(), collect.clone(), Some(state_index), &new_capture, ended_groups) {
                            return Some(result);
                        }
                    }
                    Transition::EndCapture(g) => {
                        let mut new_capture = capture_groups.clone();
                        new_capture.retain(|(x, _)| {*x == g});
                        let c = new_capture[0].1;
                        let mut new_ended = ended_groups.clone();
                        // If there already has been an ended see if it equals
                        if let Some(result) = ended_groups.get(&g) {
                            if *result != c {
                                continue;
                            }
                        }
                        // If there isn't already an ended, add an ended
                        else {
                            new_ended.insert(g, c);
                        }
                        let mut new_capture = capture_groups.clone();
                        new_capture.retain(|(x, _)| {*x != g});
                        if let Some(result) = recurse(head, p, p_consumed, f, destination as i32, state_points.clone(), collect.clone(), Some(state_index), &new_capture, &new_ended) {
                            return Some(result);
                        }

                    }
                }
            }
            // If none of those transitiions work 
            None
        }

        let mut collect: HashMap<Color, Vec<Color>> = HashMap::new();
        // Create collection bins for the consuming of colors
        self.colors.keys().for_each(|k| {
            collect.insert(*k, vec![]);
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

        // The entry point for each state
        let mut state_points = vec![Point::from(0, 0); self.states.len()];
        // Make sure to put in where you're entering!
        state_points[0] = head_pos;

        recurse(head_pos, &p, &HashSet::new(), self, 0, state_points.clone(), collect.clone(), None, &vec![], &HashMap::new())
    }

    pub fn print(&self) {
        for i in 0..self.states.len() {
            println!("{}: {:?}", i, self.states[i]);
        }
    }


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
            panic!("No function color found in Fsm definition");
        }

        FSMBuilder{
            states: vec![State::new()],
            head_pos,
            p,
            colors,
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
        let color = *self.color(c).unwrap().0;
        // ...0[]
        self.states[len - 1].t.push((len, Transition::Consume(color)));
        // ...0[Consume(1)]
        self.states.push(State::new());
        // ...0[Consume(1)], 1[]

    }

    /// Move relative to a current state/desired state
    /// p is relative coordinates to the state
    /// Adds another empty state afterwards
    fn move_rel(&mut self, s: Option<usize>, p: Point) {
        let len = self.states.len();
        // ...0[]
        self.states[len - 1].t.push((len, Transition::MoveRelative(s.unwrap_or(len - 1), p)));
        // ...0[MoveRelative(0/s)]
        self.states.push(State::new());
        // ...0[MoveRelative(0/s)], 1[]
    }

    // Starts Capture group for group g
    // Indicates that whenever something is consumed, add to g
    fn start_capture(&mut self, g: u8) {
        let len = self.states.len();
        // ...0[]
        self.states[len - 1].t.push((len, Transition::Capture(g)));
        // ...0[Capture(g)]
        self.states.push(State::new());
        // ...0[Capture(g)], 1[]
    }

    // Ends Capture group for group g
    // Indicates to stop adding to g
    fn end_capture(&mut self, g: u8) {
        let len = self.states.len();
        // ...0[]
        self.states[len - 1].t.push((len, Transition::EndCapture(g)));
        // ...0[EndCapture(g)]
        self.states.push(State::new());
        // ...0[EndCapture(g)], 1[]
    }

    // Loop group g and continue going p direction while consuming c
    // Ex loop group 8 and keep heading (0,1) {up}
    fn loop_please(&mut self, p: Point, c: Color, g: u8) {
        // Start Capture -> Epsilon(3)/MoveRel -> Consume -> End Capture/Epsilon(1)
    
        self.start_capture(g);
        // ... 0[Capture], 1[]

        let len = self.states.len();
        self.move_rel(Some(len - 1), p);
        // ... 0[Capture], 1[MoveRel(pos)], 2[]
        self.states[len - 1].t.push((len + 1, Transition::Epsilon));
        // ... 0[Capture], 1[MoveRel(pos),Epsilon(3)]
        
        self.consume(c);
        // ... 0[Capture], 1[Epsilon(3), MoveRel(pos)], 2[Consume(c)], 3[]

        let len = self.states.len();
        self.states[len - 1].t.push((len - 3, Transition::Epsilon));
        // ... 0[Capture], 1[Epsilon(3), MoveRel(pos)], 2[Consume(c)], 3[Epsilon(2)], 4[]
        self.end_capture(g);
        // ... 0[Capture], 1[Epsilon(3), MoveRel(pos)], 2[Consume(c)], 3[Epsilon(2), Capture], 4[]
    }
    
    /// Adds input color to look for
    pub fn add_input(&mut self, c: Color) {
        self.colors.insert(c, ColorType::Input);
    }

    /// Adds output color to look for
    pub fn add_output(&mut self, c: Color) {
        self.colors.insert(c, ColorType::Output);
    }
    
    /// Identifies if a selected color is significant to finite state machine
    fn color(&self, c: Color) -> Option<(&Color, &ColorType)> {
        return self.colors.get_key_value(&c);
    }

    /// Recursively build a Fsm from the Fsm Builder
    pub fn build(mut self) -> Fsm {
        self.recurse(true);
        let fsm = Fsm {
            states: std::mem::take(&mut self.states),
            colors: std::mem::take(&mut self.colors),
        };
        drop(self);
        return fsm;
    }

    // Recurses through a symbol and creates an FSM
    // Consume tag indicates whether it should consume on entering new branch
    fn recurse(&mut self, consume: bool) {
        let head_pos = self.head_pos;
        let cur_state = self.states.len() - 1;
        let head_color = self.p.get_point(self.head_pos);
        if consume {
            self.consume(head_color);
            self.p.set_point(head_pos, WHITE);
        }

        for pos in SURROUNDING {
            let next_position = head_pos + pos;
            if !self.p.in_bounds(next_position) {
                continue;
            }

            let cur_color = self.p.get(next_position.x, next_position.y);

            // SPECIAL LOOP CODE:
            // It can be black -> red as long as green and blue are 0 and red != 255
            if cur_color.g == 0 && cur_color.b == 0 && cur_color.r != 255 {
                // Gotta make sure it's at least two blacks in a row
                let mut black_count = 1;
                let mut black_pos = next_position;
                loop {
                    // Keep going the same direction 
                    black_pos += pos; 
                    if !self.p.in_bounds(black_pos) {
                        break;
                    }
                    let cur_black = self.p.get_point(black_pos);
                    // Make sure it's all the same balck
                    if cur_black == cur_color {
                        black_count += 1;
                    }
                    else {
                        break;
                    }
                }
                // If we find we have more than 1 black then we have a loop
                if black_count <= 1 {
                    continue;
                }
                // Get rid of all the blacks along the way
                for i in 0..=black_count {
                    self.p.set_point(next_position + (pos * i), WHITE);
                }
                self.loop_please(pos, head_color, cur_color.r);
                // Essentially goes to one after last black, pretend you are there already, don't
                // reconsume, and go look around
                if self.p.in_bounds(black_pos) {
                    self.head_pos = black_pos;
                    self.recurse(false);
                }
                else {
                    continue;
                }
            }
            // If we don't care about the color of the surrounding pixel go to the next one
            else if self.color(cur_color).is_none() {
                continue;
            }
            // NORMAL CASE:
            else {
                self.move_rel(Some(cur_state), pos);
                self.head_pos = next_position;
                self.recurse(true);
            }
        } 
    }
}


#[cfg(test)]
mod tests {
    use crate::tokenizer::picture;
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
            let fsm_builder = Fsm::builder(p);
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
            let fsm_builder = Fsm::builder(p.clone());
            let fsm = fsm_builder.build();
            assert!(fsm.identify(p.clone()).is_some());
        }
    }
    #[test]
    /// Checks that all of the fsm definition tests compile into an fsm regardless of correctness
    fn loop_fsm_compiles() {
        let paths = fs::read_dir("./tests/loop_definitions").unwrap();
        for path in paths {
            let p = picture::Picture::open_pic(path.unwrap().path().to_str().unwrap());
            let fsm_builder = Fsm::builder(p);
            let fsm = fsm_builder.build();
            assert!(fsm.states.len() > 1);
        }
    }
}
