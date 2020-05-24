use {
  std::env,
  std::fmt::Display,
  serde::{Deserialize, Serialize},
  nom::{
    IResult,
    error::{ErrorKind, ParseError},
    // bytes::{
    //   complete::{take_while},
    // },
    character::{
      is_alphanumeric, 
      complete::{
        // char,
        not_line_ending, line_ending,
        none_of, one_of,
        space0, space1,
      },
    },
    combinator::{map, map_opt, map_res, value, verify, rest},
    sequence::{delimited, preceded, tuple},
    char, tag, is_not, take_while_m_n, take_while, take, take_while1, take_str,// re_find,
    many0, many1_count,
  },
};

// named!(escape, delimited!(tag!("{{{"), is_not!("}}}"), tag!("}}}")));
// named!(line, take_while!(is_not_line_end));
// named!(space, take_while!(is_space));
named!(take1, take!(1));
named!(bold<&str, &str>, delimited!(tag!("**"), is_not!("**"), tag!("**")));
named!(italic<&str, &str>, delimited!(tag!("//"), is_not!("//"), tag!("//")));
// named!(text<&str, &str, CreoleErr>, take_while1!(|c| !INLINE_MARKUP_STARTS.contains(c) ));
named!(linebreak<&str, &str>, tag!("\\\\"));
named!(link<&str, &str>, delimited!(tag!("[["), is_not!("]]"), tag!("]]")));
named!(image<&str, &str>, delimited!(tag!("{{"), is_not!("}}"), tag!("}}")));
named!(numberedlist<&str, &str>, terminated!(take_while1!(is_numlist), char!(' ')));
named!(bulletlist<&str, &str>, terminated!(take_while1!(is_bulletlist), char!(' ')));
named!(heading<&str, &str>, terminated!(take_while_m_n!(2, 7, is_heading), char!(' ')));
named!(horizontal<&str, &str>, tag!("----"));
fn is_markup(c: char) -> bool {
  MARKUPS.contains(c)
}
fn is_not_markup(c: char) -> bool {
  // !MARKUPS.contains(c)
  !"*/#=[]{}-|\\".contains(c)
}
// fn is_not_line_end(c: u8) -> bool {
//   c as char != '\n' && c as char != '\r'
// }
fn is_not_line_end(c: char) -> bool {
  c != '\n' && c != '\r'
}
fn is_asterisk(c: char) -> bool {
  c == '*'
}
fn is_numlist(c: char) -> bool {
  c == '#'
}
fn is_bulletlist(c: char) -> bool {
  c == '*'
}
fn is_heading(c: char) -> bool {
  c == '='
}
fn is_space(c: char) -> bool {
  c != ' '
}
fn non_markup(s:&str) -> IResult<&str, &str> {
  nom::bytes::complete::take_while(is_not_markup)(s)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Creole<T:Into<String>+Sized>{
  Text(T),
  Bold(T),
  Italic(T),
  BulletList(u8, T),
  NumberedList(u8, T),
  Link(T, T),
  Heading(u8, T),
  Linebreak,
  HorizontalLine,
  Image(T, T),
  TableHeaderCell(T),
  TableRowCell(T),
  // Escape(T),
}
// type ParsedCreoles<T> = Vec<Creole<T>>;
pub struct ParsedCreoles<T:Into<String>+Sized>(Vec<Creole<T>>);
impl <T:Into<String>+Sized> From<ParsedCreoles<T>> for Vec<Creole<T>> {
  fn from(c: ParsedCreoles<T>) -> Vec<Creole<T>> {
    c.0
  }
}
impl<'a> From<ParsedCreoles<String>> for Vec<Creole<&'a str>> {
  fn from(v: ParsedCreoles<String>) -> Vec<Creole<&'a str>> {
    v.into()
  }
}
// impl From<Creole<String>> for Creole<&str> {
//   fn from(c: Creole<String>) -> Self {
//     match c {
//       Creole::Text(t) => Creole::Text(&t.clone()),
//       Creole::Bold(t) => Creole::Bold(&t),
//       Creole::Italic(t) => Creole::Italic(&t),
//       Creole::BulletList(i, t) => Creole::BulletList(i, &t),
//       Creole::NumberedList(i, t) => Creole::BulletList(i, &t),
//       Creole::Link(t, tt) => Creole::Link(&t, &tt),
//       Creole::Heading(i, t) => Creole::Heading(i, &t),
//       Creole::Linebreak => Creole::Linebreak,
//       Creole::HorizontalLine => Creole::HorizontalLine,
//       Creole::Image(t, tt) => Creole::Image(&t, &tt),
//       Creole::TableHeaderCell(t) => Creole::TableHeaderCell(&t),
//       Creole::TableRowCell(t) => Creole::TableRowCell(&t),
//     }
//   }
// }
// impl From<&Creole<String>> for Creole<&str> {
//   fn from(c: &Creole<String>) -> Self {
//     Creole::<&str>::from(*c)
//   }
// }
impl From<Creole<&str>> for Creole<String> {
  fn from(c: Creole<&str>) -> Self {
    match c {
      Creole::Text(t) => Creole::Text(String::from(t)),
      Creole::Bold(t) => Creole::Bold(String::from(t)),
      Creole::Italic(t) => Creole::Italic(String::from(t)),
      Creole::BulletList(i, t) => Creole::BulletList(i, String::from(t)),
      Creole::NumberedList(i, t) => Creole::BulletList(i, String::from(t)),
      Creole::Link(t, tt) => Creole::Link(String::from(t), String::from(tt)),
      Creole::Heading(i, t) => Creole::Heading(i, String::from(t)),
      Creole::Linebreak => Creole::Linebreak,
      Creole::HorizontalLine => Creole::HorizontalLine,
      Creole::Image(t, tt) => Creole::Image(String::from(t), String::from(tt)),
      Creole::TableHeaderCell(t) => Creole::TableHeaderCell(String::from(t)),
      Creole::TableRowCell(t) => Creole::TableRowCell(String::from(t)),
    }
  }
}
impl From<&Creole<&str>> for Creole<String> {
  fn from(c: &Creole<&str>) -> Self {
    Creole::<String>::from(*c)
  }
}
impl PartialEq<Creole<String>> for Creole<&str> {
  fn eq(&self, other:&Creole<String>)-> bool {
    let s = Creole::<String>::from(self);
    &s == other
  }
}
// impl PartialEq<ParsedCreoles<String>> for ParsedCreoles<&str> {
//   fn eq(&self, other:&ParsedCreoles<String>)-> bool {
//     for i in {
//       if self == i { return true }
//     }
//     false
//   }
// }

#[derive(Debug, Clone, PartialEq)]
pub enum CreoleErr{
  None
}

impl ParseError<&str> for CreoleErr {
  fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
    Self::None
  }
  fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
    Self::None
  }
}

const MARKUPS :&'static str = "*/#=[]{}-|\\";
const INLINE_MARKUP_STARTS :&'static str = "*/[{\\";
// const INLINE_MARKUP_ENDS :&'static str = "]}";

fn creole_line<'a>(i:&'a str) -> IResult<&'a str, Vec<Creole<String>>> {
  debug!("line:{}", i);
  let mut len :usize = i.len();
  if len == 0 { return Ok((&i[len..len], vec![])) }

  // single item per line
  if let Ok((i, b)) = bulletlist(i){
    debug!("bulletlist:{}:{} , left:{}:{}", b.len(), b, i.len(), i);
    return Ok((i, vec![Creole::BulletList(b.len() as u8-1, i.to_string())]))
  }
  if let Ok((i, b)) = numberedlist(i){
    debug!("numberedlist:{}:{}, left:{}:{}", b.len(), b, i.len(), i);
    return Ok((i, vec![Creole::NumberedList(b.len() as u8-1, i.to_string())]))
  }
  if let Ok((i, b)) = heading(i){
    debug!("heading:{}:{}, left:{}:{}", b.len(), b, i.len(), i);
    return Ok((i, vec![Creole::Heading(b.len() as u8-2, i.to_string())]))
  }
  if let Ok((i, b)) = horizontal(i){
    debug!("horizontal line");
    return Ok((i, vec![Creole::HorizontalLine]))
  }
  
  debug!("multi item per line left:{}", i);
  if i.len() == 0 {
    return Ok((i, vec![]))
  }
  // multi item per line
  let mut v :Vec<Creole<_>> = vec![];
  // if cnt < len {
    let r : IResult<&'a str, Vec<Creole<_>>> = nom::multi::many1::<&'a str, Creole<_>, _, _>(|i:&'a str| -> IResult<&'a str, Creole<String>> {
      let len :usize = i.len();
      if len == 0 {return Err(nom::Err::Error((i, ErrorKind::Many1)))} // done
      debug!("text left:{}:{}", len, i);
      let text = nom::bytes::complete::take_while1::<_, &str, CreoleErr>(|c| !INLINE_MARKUP_STARTS.contains(c) );

      if let Ok((i, b)) = text(i) {
        debug!("normal text:{}:{}:{}:{}", b.len(), b, i.len(), i);
        Ok((i, Creole::Text(b.to_string())))
      } else if let Ok((i, b)) = bold(i) {
        debug!("bold:{}", b);
        Ok((i, Creole::Bold(b.to_string())))
      } else if let Ok((i, b)) = italic(i) {
        debug!("italic:{}", b);
        Ok((i, Creole::Italic(b.to_string())))
      } else if let Ok((i, b)) = linebreak(i) {
        debug!("linebreak");
        Ok((i, Creole::Linebreak))
      } else {
        debug!("unfinished markup:{}:{}", i.len(), i);
        Ok((&i[1..], Creole::Text((&i[..1]).to_string())))
        // Err(nom::Err::Error((i, ErrorKind::Many1)))
      }
    })(i);
    // let r : IResult<&str, Vec<Creole>> = nom::multi::many0(|i:&'a str| -> IResult<&'a str, Creole<'a>> {
    //   match non_markup(i) {
    //     Ok((i, b)) => Ok((i, Creole::Text(b))),
    //     Err(e) => Err(e)
    //   }
    // })(i);
    // let r : IResult<&str, Vec<&str>> = texts(i);
    debug!("after many1 finish:{:?}, {}", r, i);
    if let Ok((i, vv)) = r{
      // for vvv in vv {
        // v.push(Creole::Text(vvv));
      // }
      // v.push(Creole::Text(vv));
      v.extend(vv);
    }
    // let r = bold(i);
    // if let Ok((i, vv)) = r{
    //   v.push((Creole::Bold(vv)))
    // }
  //   Ok((&i[len..], v))
  // }else{
  //   Ok((i, v))
  // }
  Ok((i, v))
}
fn _line_ending(input:&str) -> IResult<&str, &str> {
  line_ending(input)
}

/// parser top entry point
pub fn creoles<'a>(i:&'a str) -> IResult<&'a str,Vec<Creole<String>>> {
  let mut v = vec![];
  // let (mut start, mut end) :(usize, usize) = (0, 0);
  let mut start = 0usize;
  let end = i.len();
  loop {
    let (i, t) = not_line_ending(&i[start..])?;
    let tl = t.len();
    debug!("not_line_ending:{}", tl);
    if tl>0 {
      start += tl;
      let (_, vv) = creole_line(/* v.last(),  */t)?;
      v.extend(vv);
    } else {
      debug!("double line ending");
      v.push(Creole::Linebreak);
    }
    start += {
      let c = match _line_ending(i){
        Ok((_, c)) => c.len(),
        // Err(Err::Error((c,_))) => 0,
        _ => 0,
      };
      debug!("line ending:{}", c);
      c
    };
    if start >= end { debug!("parse finished"); break }
  }
  // let (i, l) = take_while(is_not_line_end)(i)?;
  // v.push(line(l));
  // let (i, rr) = line(i);
  // v.extend(rr);
  let (i, r) = rest(i)?;
  Ok((i, v))
}

// #[test]
// fn tests() {
//   assert_eq!(text("a"), Ok(("", "a")));
// }

#[test]
fn text_tests() {
  assert_eq!(creoles("ab1"), Ok(("", vec![Creole::Text("ab1").into()])));
  // assert_eq!(creoles("ab1").unwrap().1, Vec::<Creole::<String>>::from(vec![Creole::Text("ab1")]));
}

#[test]
fn text_style_tests(){
  assert_eq!(creoles("**a**"), Ok(("", vec![Creole::Bold("a").into()])));
  assert_eq!(creoles("//a//"), Ok(("", vec![Creole::Italic("a").into()])));
  assert_eq!(creoles("a**b**//c//d"), Ok(("", vec![Creole::Text("a").into(), Creole::Bold("b").into(), Creole::Italic("c").into(), Creole::Text("d").into()])));
}

#[test]
fn linebreak_tests(){
  assert_eq!(creoles("a\nb\n\nc"), Ok(("", vec![Creole::Text("a").into(), Creole::Text("b").into(), Creole::Linebreak, Creole::Text("c").into()])));
  assert_eq!(creoles("a\\\\b"), Ok(("", vec![Creole::Text("a").into(), Creole::Linebreak, Creole::Text("b").into()])));
  assert_eq!(creoles("a\\b"), Ok(("", vec![Creole::Text("a").into(), Creole::Text("\\").into(), Creole::Text("b").into()])));
}

#[test]
fn list_tests() {
  assert_eq!(creoles("* a"), Ok(("", vec![Creole::BulletList(0, "a").into()])));
  assert_eq!(creoles("** b"), Ok(("", vec![Creole::BulletList(1, "b").into()])));
  assert_eq!(creoles("*** c"), Ok(("", vec![Creole::BulletList(2, "c").into()])));

  assert_eq!(creoles("# a"), Ok(("", vec![Creole::NumberedList(0, "a").into()])));
  assert_eq!(creoles("## b"), Ok(("", vec![Creole::NumberedList(1, "b").into()])));
  assert_eq!(creoles("### c"), Ok(("", vec![Creole::NumberedList(2, "c").into()])));
}

#[test]
fn heading_tests(){
  assert_eq!(creoles("== a"), Ok(("", vec![Creole::Heading(0, "a").into()])));
  assert_eq!(creoles("=== b"), Ok(("", vec![Creole::Heading(1, "b").into()])));
  assert_eq!(creoles("==== c"), Ok(("", vec![Creole::Heading(2, "c").into()])));
}

// #[test]
// fn link_tests(){
//   assert_eq!(creoles("[[a]] "), Ok(("", vec![Creole::Link("", "a").into()])));
//   assert_eq!(creoles("[[https://google.com|google]] "), Ok(("", vec![Creole::Link("https://google.com", "google")])));
// }

#[test]
fn other_tests(){
  assert_eq!(creoles("----"), Ok(("", vec![Creole::HorizontalLine])));
  assert_eq!(creoles("a\n----\nb"), Ok(("", vec![Creole::Text("a").into(), Creole::HorizontalLine, Creole::Text("b").into()])));
//   assert_eq!(creoles("{{a.jpg|b}}"), Ok(("", vec![Creole::Image("a.jpg", "b").into()])));
//   assert_eq!(creoles("|=|=a|=b|\n|0|1|2|\n|3|4|5|"), Ok(("", vec![Creole::TableHeaderCell(""), Creole::TableHeaderCell("a"), Creole::TableHeaderCell("b"), Creole::TableRowCell("0"), Creole::TableRowCell("1"), Creole::TableRowCell("2"), Creole::TableRowCell("3"), Creole::TableRowCell("4"), Creole::TableRowCell("5")])));
//   assert_eq!(creoles("{{{\n== [[no]]:\n//**don't** format//\n}}}"), Ok(("", vec![Creole::Text("== [[no]]:\n//**don't** format//")])));
}