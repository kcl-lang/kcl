struct Square(f32);
struct Rectangle(f32, f32);
trait Area {
    fn get_area(&self) -> f32;
}
impl Area for Square {
    fn get_area(&self) -> f32 {
        100.0
    }
}
impl Area for Rectangle {
    fn get_area(&self) -> f32 {
        20.0
    }
}
fn main() {
    let c = Box::new(Square(3f32));
    let s: Box<dyn Area> = c;
    // c.get_area();
}
