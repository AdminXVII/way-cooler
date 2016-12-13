use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref};
use rustwlc::{Geometry, Size, Point};
use cairo::{Context, ImageSurface, Format, Operator, Status, SolidPattern};

pub trait Border {
    /// Renders the border around the geometry of a view.
    fn render(&self, view_g: Geometry);

    fn get_color(&self) -> Color;

    fn get_context(&self) -> &Context;

    /// Draws the line from (x, y) to (x+width,y+height) where width/height.
    ///
    /// You should just use the default implementation in most cases.
    fn draw_line(&self, mut x: f64, mut y: f64, mut w: f64, mut h: f64) {
        let Color { red, green, blue} = self.get_color();
        let pattern = SolidPattern::from_rgb(red as f64 / 255.0,
                                             green as f64 / 255.0,
                                             blue as f64 / 255.0);
        let context = self.get_context();
        context.set_source(&pattern);
        if w > 1.0 && h > 1.0 {
            context.rectangle(x, y, w, h);
            context.fill();
        } else {
            if w == 1.0 {
                x += 0.5;
                h += y;
                w = x;
            }

            if h == 1.0 {
                y += 0.5;
                w += x;
                h = y;
            }

            context.move_to(x, y);
            context.set_line_width(1.0);
            context.line_to(w, h);
            context.stroke();
        }
    }
}

#[derive(Clone)]
/// A border for a container.
pub struct BaseBorder {
    context: Context,
    thickness: u32,
    color: Color,
    geometry: Geometry
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopBorder {
    border: BaseBorder
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottomBorder {
    border: BaseBorder
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RightBorder {
    border: BaseBorder
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeftBorder {
    border: BaseBorder
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Color {
    red: u32,
    green: u32,
    blue: u32
}

impl Color {
    pub fn new(red: u32, green: u32, blue: u32) -> Self {
        Color { red: red, green: green, blue: blue}
    }
}

impl BaseBorder {
    pub fn new(geometry: Geometry, thickness: u32, color: Color) -> Self {
        let Size { w, h } = geometry.size;
        let h = h as i32;
        let w = w as i32;
        let stride = calculate_stride(w as u32) as i32;
        let data: Vec<u8> = iter::repeat(0).take(w as usize * stride as usize).collect();
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer, drop_data, Format::ARgb32, h, w, stride);
        let context = Context::new(&surface);
        context.set_operator(Operator::Source);
        match context.status() {
            Status::Success => {},
            err => {
                panic!("Cairo context failed with {:#?}", err);
            }
        }
        BaseBorder {
            context: context,
            thickness: thickness,
            color: color,
            geometry: geometry
        }
    }
}

impl Debug for BaseBorder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Debug")
            .field("thickness", &self.thickness as &Debug)
            .field("color", &self.color as &Debug)
            .finish()
    }
}

impl PartialEq for BaseBorder {
    fn eq(&self, other: &BaseBorder) -> bool {
        (self.thickness == other.thickness && self.color == other.color
         && self.geometry == other.geometry)
    }
}

impl Eq for BaseBorder {}

unsafe impl Send for BaseBorder {}
unsafe impl Sync for BaseBorder {}

impl BottomBorder {
    fn new(geometry: Geometry, thickness: u32, color: Color) -> Self {
        BottomBorder { border: BaseBorder::new(geometry, thickness, color)}
    }

}

impl Border for BottomBorder {
    fn render(&self, view_g: Geometry) {
        let Point { x, y } = self.geometry.origin;
        let top_border_height = view_g.origin.y - self.geometry.origin.y;
        let height = self.geometry.size.h - (top_border_height as u32 + view_g.size.h);
        if height > 0 {
            self.draw_line(x as f64,
                        (y as u32 + self.geometry.size.h - height) as f64,
                        self.geometry.size.w as f64,
                        height as f64);
        }
    }

    fn get_context(&self) -> &Context {
        &self.context
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

impl TopBorder {
    fn new(geometry: Geometry, thickness: u32, color: Color) -> Self {
        TopBorder { border: BaseBorder::new(geometry, thickness, color)}
    }

}

impl Border for TopBorder {
    fn render(&self, view_g: Geometry) {
        let Point { x, y } = self.geometry.origin;
        let height = view_g.origin.y - self.geometry.origin.y;
        if height > 0 {
            self.draw_line(x as f64, y as f64, self.geometry.size.w as f64, height as f64);
        }
    }

    fn get_context(&self) -> &Context {
        &self.context
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

impl RightBorder {
    fn new(geometry: Geometry, thickness: u32, color: Color) -> Self {
        RightBorder { border: BaseBorder::new(geometry, thickness, color)}
    }
}

impl Border for RightBorder {

    fn render(&self, view_g: Geometry) {
        let Point { x, y } = self.geometry.origin;
        let left_border_width = (view_g.origin.x - self.geometry.origin.x) as u32;
        let width = self.geometry.size.w - view_g.size.w - left_border_width;
        if width > 0 {
            self.draw_line((x as u32 + self.geometry.size.w - width) as f64,
                           y as f64,
                           width as f64,
                           self.geometry.size.h as f64);
        }
    }

    fn get_context(&self) -> &Context {
        &self.context
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

impl LeftBorder {
    fn new(geometry: Geometry, thickness: u32, color: Color) -> Self {
        LeftBorder { border: BaseBorder::new(geometry, thickness, color)}
    }
}

impl Border for LeftBorder {

    fn render(&self, view_g: Geometry) {
        let Point { x, y } = self.geometry.origin;
        let width = view_g.origin.x - self.geometry.origin.x;
        self.draw_line(x as f64, y as f64, width as f64, self.geometry.size.h as f64);
    }

    fn get_context(&self) -> &Context {
        &self.context
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

impl Deref for TopBorder {
    type Target = BaseBorder;

    fn deref(&self) -> &BaseBorder {
        &self.border
    }
}

impl Deref for BottomBorder {
    type Target = BaseBorder;

    fn deref(&self) -> &BaseBorder {
        &self.border
    }
}
impl Deref for RightBorder {
    type Target = BaseBorder;

    fn deref(&self) -> &BaseBorder {
        &self.border
    }
}
impl Deref for LeftBorder {
    type Target = BaseBorder;

    fn deref(&self) -> &BaseBorder {
        &self.border
    }
}

/// Calculates the stride for ARgb32 encoded buffers
fn calculate_stride(width: u32) -> u32 {
    // function stolen from CAIRO_STRIDE_FOR_WIDTH macro in cairoint.h
    // Can be found in the most recent version of the cairo source
    (32 * width + 7) / 8 + 4 & !4
}

#[allow(dead_code)]
fn drop_data(_: Box<[u8]>) {}


#[cfg(tests)]
mod tests {
    use super::calculate_stride;

    #[test]
    fn test_calculate_stride() {
        let test_data = [
            (100, 400),
            (200, 800),
            (300, 1200),
            (400, 1600),
            (500, 2000),
            (600, 2400),
            (700, 2800),
            (800, 3200),
            (900, 3600)
        ];
        for &(width, stride) in test_data.iter() {
            assert_eq!(calculate_stride(width), stride);
        }
    }
}
