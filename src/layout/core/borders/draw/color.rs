/// Color to draw to the screen, including the alpha channel.
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8
}


impl Color {
    /// Makes a new solid color, with no transparency.
    pub fn solid_color(red: u8, green: u8, blue: u8) -> Self {
        Color {
            red: red,
            green: green,
            blue: blue,
            alpha: 255
        }
    }
}
