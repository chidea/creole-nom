fn hello_parser(i: &str) -> nom::IResult<&str, &str> {
	nom::bytes::complete::tag("hello")(i)
}

fn main() {
	println!("{:?}", hello_parser("hello"));
	println!("{:?}", hello_parser("hello world"));
	println!("{:?}", hello_parser("goodbye hello again"));
}