#![allow(non_snake_case)]
mod picture;
mod fsm;
mod point;

use picture::Picture;
use fsm::FSM;
use picture::Color;
use std::fs;

fn main() {
    let picture = Picture::open_pic("test.png");
    let mut fsm = FSM::builder(picture.clone());
    fsm.add_input(Color::from(255, 0, 0));
    let f = fsm.build();
    f.print();
    print!("{:?}", f.identify(picture.clone()));
}
