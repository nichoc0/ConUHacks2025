fn main() {
    // Integer division (truncates toward zero)
    let integer_result = -5 / 3;
    println!("Integer division: {}", integer_result);
    
    // Floating-point division (precise result)
    let float_result = -5.0 / 3.0;
    println!("Floating-point division: {:.2}", float_result);
    
    // Alternative using type casting
    let cast_result = -5 as f64 / 3 as f64;
    println!("Type cast division: {:.3}", cast_result);
}
