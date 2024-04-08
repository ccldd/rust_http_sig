use http::HeaderName;

fn main() {
    println!("Hello world");
    let hn: HeaderName = HeaderName::from_static("authorization");
    println!("{}", hn);
}
