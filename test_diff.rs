use rat::utils::diff::DiffGenerator;

fn main() {
    let original = "# I Love You";
    let modified = "# I Love You\n\nThis is a simple markdown file expressing love and appreciation.";
    
    let diff = DiffGenerator::generate_diff(original, modified);
    println!("{}", diff);
}