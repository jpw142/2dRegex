#![allow(non_snake_case)]
mod picture;
mod fsm;
mod point;

use picture::Picture;
use fsm::Fsm;
use picture::Color;
// use std::fs;


fn main() {
    //let picture = Picture::open_pic("test.png");
    //let mut fsm = Fsm::builder(picture.clone());
    //fsm.add_input(Color::from(255, 0, 0));
    //let f = fsm.build();
    //f.print();
    //print!("{:?}", f.identify(picture.clone()));
    //println!("");

    let picture2 = Picture::open_pic("loop_test.png");
    let picture3 = Picture::open_pic("loop_test2.png");
    let mut fsm = Fsm::builder(picture2.clone());
    fsm.add_input(Color::from(255, 0, 0));
    let f = fsm.build();
    f.print();
    print!("{:?}", f.identify(picture3.clone()));
}
