#![allow(dead_code)]
use image::io::Reader;
use crate::point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn from(r: u8, g: u8, b: u8) -> Self {
        return Color{
            r,
            g,
            b,
        }
    }
}


pub const RED: Color = Color{r: 255, g: 0, b: 0};
pub const GREEN: Color = Color{r: 0, g: 255, b: 0};
pub const BLUE: Color = Color{r: 0, g: 0, b: 255};
pub const YELLOW: Color = Color{r: 255, g: 255, b: 0};
pub const WHITE: Color = Color{r: 255, g: 255, b: 255};
pub const BLACK: Color = Color{r: 0, g: 0, b: 0};


#[derive(Debug, Clone)]
pub struct Picture {
    pub pixels: Vec<Color>,
    pub width: i32,
    pub height: i32,
}

impl Picture {
    /// Opens image if the file path is correct, returns the image in the picture format
    pub fn open_pic(path: &str) -> Picture {
        // Opens image
        let img = Reader::open(path).unwrap().decode().unwrap().to_rgb8();

        // Sets the measurements of the picture
        let width = img.width();
        let height = img.height();

        let data: Vec<u8>= img.into_raw();

        let mut ret_img: Picture = Picture{
            pixels: vec![],
            width: width as i32,
            height: height as i32,
        };

        // Add all the pixels to the picture
        for c in 0..(height * width) {
            let c3 = (c * 3) as usize;
            let color = Color { r: data[c3], g: data[c3 + 1], b: data[c3 + 2]}; 
            ret_img.pixels.push(color);
        }

        return ret_img;
    }

    /// Returns whether or not a point is within the bounds of a picture
    pub fn in_bounds(&self, p: Point) -> bool {
        return p.x < self.width && p.x >= 0 && p.y < self.height && p.y >= 0
    }

    /// Gets a pixel's color
    pub fn get(&self, x: i32, y: i32) -> Color {
        return self.pixels[(y * self.width + x) as usize]
    }

    /// Sets a pixels color
    pub fn set(&mut self, x: i32, y: i32, c: Color) {
        self.pixels[(y * self.width + x) as usize] = c; 
    }

    /// Returns a subpicture, inclusive bounds, from (x1, y1) -> (x2, y2)
    pub fn subpicture(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Picture {
        let mut vec = vec![];
        for j in y1..=y2 {
            for i in x1..=x2 {
                vec.push(self.get(i, j));
            }
        }
        return Picture {
            pixels: vec,
            width: (x2 - x1) + 1,
            height: (y2 - y1) + 1,
        }

    }

    /// Rotates picture clockwise
    pub fn rotate(&self) -> Picture {
        let mut new_picture = Picture{width: self.height, height: self.width, pixels: vec![WHITE; self.pixels.len()]};
        for i in 0..self.width {
            for j in 0..self.height {
                new_picture.set(j, i, self.get(i, j));
            }
        }
        return new_picture;
    }

    /// Returns a color if all 4 corners of the picture are the same color
    pub fn four_corners(&self) -> Option<Color> {
        let one = self.get(0,0);
        let two = self.get(self.width - 1, 0);
        let three = self.get(self.width - 1, self.height - 1);
        let four = self.get(0, self.height - 1);
        if one != two || two != three || three != four {
            return None
        }
        if one == WHITE {
            return None;
        }
        return Some(one);
    }
}
