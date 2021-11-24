use wgslsmith::generator::Generator;
use wgslsmith::types::TypeConstraints;

fn main() {
    println!("{}", Generator::new().gen_expr(TypeConstraints::INT));
}
