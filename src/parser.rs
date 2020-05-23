use {
  std::env,
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
named!(heading<&str, &str>, terminated!(take_while_m_n!(2, 6, is_heading), char!(' ')));
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
fn non_markup(s: &str) -> IResult<&str, &str> {
  nom::bytes::complete::take_while(is_not_markup)(s)
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Creole<T:std::string::ToString>{
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

#[derive(Debug, Clone, PartialEq)]
pub enum CreoleErr {
  None,
}

impl ParseError<T:ToString> for CreoleErr {
  fn from_error_kind(input: T, kind: ErrorKind) -> Self {
    Self::None
  }
  fn append(input: T, kind: ErrorKind, other: Self) -> Self {
    Self::None
  }
}

const MARKUPS: &'static str = "*/#=[]{}-|\\";
const INLINE_MARKUP_STARTS: &'static str = "*/[{\\";
// const INLINE_MARKUP_ENDS :&'static str = "]}";

/// debug purpose (because Creole<&str> is easier to build than Creole<String>)
fn _creoles(i:&str) -> IResult<&str,Vec<Creole<&str>>> {
  creoles::<&str>(i)
}

// external creole parsers (String)
/// parser top entry point
pub fn creoles<T:ToString>(i:&str) -> IResult<&str,Vec<Creole<T>>> {
  let mut v = vec![];
  // let (mut start, mut end) :(usize, usize) = (0, 0);
  let mut start = 0usize;
  let end = i.len();
  loop {
    let (i, t) = not_line_ending(&i[start..])?;
    // debug!("not_line_ending:{}", t);
    let tl = t.len();
    debug!("not_line_ending:{}", tl);
    if tl > 0 {
      start += tl;
      let (_, vv) = creole_dbg(/* v.last(),  */ t)?;
      v.extend(vv);
    } else {
      debug!("double line ending");
      v.push(Creole::Linebreak);
    }
    start += {
      let c = match _line_ending(i) {
        Ok((_, c)) => c.len(),
        // Err(Err::Error((c,_))) => 0,
        _ => 0,
      };
      debug!("line ending:{}", c);
      c
    };
    if start >= end {
      debug!("parse finished");
      break;
    }
  }
  // let (i, l) = take_while(is_not_line_end)(i)?;
  // v.push(line(l));
  // let (i, rr) = line(i);
  // v.extend(rr);
  let (i, r) = rest(i)?;
  Ok((i, v))
}

fn creole_dbg<T:ToString, 'a>(dbg:bool, i:&'a str) -> IResult<&'a str, Vec<Creole<T>>> {
  if dbg {println!("line:{}", i);}
  let mut len :usize = i.len();
  if len == 0 { return Ok((&i[len..len], vec![])) }

  // single item per line
  if let Ok((i, b)) = bulletlist(i){
    if dbg {println!("bulletlist:{}:{} , left:{}:{}", b.len(), b, i.len(), i)}
    return Ok((i, vec![Creole::BulletList(b.len() as u8-1, i)]))
  }
  if let Ok((i, b)) = numberedlist(i){
    if dbg {println!("numberedlist:{}:{}, left:{}:{}", b.len(), b, i.len(), i)}
    return Ok((i, vec![Creole::NumberedList(b.len() as u8-1, i)]))
  }
  if let Ok((i, b)) = heading(i){
    if dbg {println!("heading:{}:{}, left:{}:{}", b.len(), b, i.len(), i)}
    return Ok((i, vec![Creole::Heading(b.len() as u8-2, i)]))
  }
  if let Ok((i, b)) = horizontal(i){
    if dbg {println!("horizontal line")}
    return Ok((i, vec![Creole::HorizontalLine]))
  }
  
  if dbg {println!("multi item per line left:{}", i)}
  if i.len() == 0 {
    return Ok((i, vec![]))
  }
  // multi item per line
  let mut v :Vec<Creole> = vec![];
  // if cnt < len {
    let r : IResult<&str, Vec<Creole<T>>> = nom::multi::many1::<&str, Creole<T>, _, _>(|i:&'a str| -> IResult<&'a str, Creole<T>> {
      let len :usize = i.len();
      if len == 0 {return Err(nom::Err::Error((i, ErrorKind::Many1)))} // done
      if dbg {println!("text left:{}:{}", len, i)}
      let text = nom::bytes::complete::take_while1::<_, &str, CreoleErr>(|c| !INLINE_MARKUP_STARTS.contains(c) );

      if let Ok((i, b)) = text(i) {
        if dbg {println!("normal text:{}:{}:{}:{}", b.len(), b, i.len(), i)}
        Ok((i, Creole::Text(b)))
      } else if let Ok((i, b)) = bold(i) {
        if dbg {println!("bold:{}", b)}
        Ok((i, Creole::Bold(b)))
      } else if let Ok((i, b)) = italic(i) {
        if dbg {println!("italic:{}", b)}
        Ok((i, Creole::Italic(b)))
      } else if let Ok((i, b)) = linebreak(i) {
        if dbg {println!("linebreak")}
        Ok((i, Creole::Linebreak))
      } else {
        if dbg {println!("unfinished markup:{}:{}", i.len(), i)}
        Ok((&i[1..], Creole::Text(&i[..1])))
        // Err(nom::Err::Error((i, ErrorKind::Many1)))
      }
    })(i);
    // let r : IResult<&str, Vec<Creole<T>>> = nom::multi::many0(|i:&'a str| -> IResult<&'a str, Creole<T>> {
    //   match non_markup(i) {
    //     Ok((i, b)) => Ok((i, Creole::Text(b))),
    //     Err(e) => Err(e)
    //   }
    // })(i);
    // let r : IResult<&str, Vec<&str>> = texts(i);
    if dbg {println!("after many1 finish:{:?}, {}", r, i)}
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

// #[test]
// fn tests() {
//   assert_eq!(text("a"), Ok(("", "a")));
// }

#[cfg(test)]
mod tests {
  use super::*;

  fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn text_tests() {
    init ();

    assert_eq!(_creoles("ab1"), Ok(("", vec![Creole::Text("ab1")])));
  }

  #[test]
  fn text_style_tests() {
    init ();

    assert_eq!(_creoles("**a**"), Ok(("", vec![Creole::Bold("a")])));
    assert_eq!(_creoles("//a//"), Ok(("", vec![Creole::Italic("a")])));
    assert_eq!(
      _creoles("a**b**//c//d"),
      Ok((
        "",
        vec![
          Creole::Text("a"),
          Creole::Bold("b"),
          Creole::Italic("c"),
          Creole::Text("d")
        ]
      ))
    );
  }

  #[test]
  fn linebreak_tests() {
    init ();

    assert_eq!(
      _creoles("a\nb\n\nc"),
      Ok((
        "",
        vec![
          Creole::Text("a"),
          Creole::Text("b"),
          Creole::Linebreak,
          Creole::Text("c")
        ]
      ))
    );
    assert_eq!(
      _creoles("a\\\\b"),
      Ok((
        "",
        vec![Creole::Text("a"), Creole::Linebreak, Creole::Text("b")]
      ))
    );
  }

  #[test]
  fn list_tests() {
    init ();

    assert_eq!(_creoles("* a"), Ok(("", vec![Creole::BulletList(0, "a")])));
    assert_eq!(_creoles("** b"), Ok(("", vec![Creole::BulletList(1, "b")])));
    assert_eq!(_creoles("*** c"), Ok(("", vec![Creole::BulletList(2, "c")])));

    assert_eq!(_creoles("# a"), Ok(("", vec![Creole::NumberedList(0, "a")])));
    assert_eq!(
      _creoles("## b"),
      Ok(("", vec![Creole::NumberedList(1, "b")]))
    );
    assert_eq!(
      _creoles("### c"),
      Ok(("", vec![Creole::NumberedList(2, "c")]))
    );
  }

  #[test]
  fn heading_tests() {
    init ();

    assert_eq!(_creoles("== a"), Ok(("", vec![Creole::Heading(0, "a")])));
    assert_eq!(_creoles("=== b"), Ok(("", vec![Creole::Heading(1, "b")])));
    assert_eq!(_creoles("==== c"), Ok(("", vec![Creole::Heading(2, "c")])));
  }

  // #[test]
  // fn link_tests(){
  //   assert_eq!(_creoles("[[a]] "), Ok(("", vec![Creole::Link("", "a")])));
  //   assert_eq!(_creoles("[[https://google.com|google]] "), Ok(("", vec![Creole::Link("https://google.com", "google")])));
  // }

  #[test]
  fn other_tests() {
    init ();

    assert_eq!(_creoles("----"), Ok(("", vec![Creole::HorizontalLine])));
    assert_eq!(
      _creoles("a\n----\nb"),
      Ok((
        "",
        vec![Creole::Text("a"), Creole::HorizontalLine, Creole::Text("b")]
      ))
    );
    //   assert_eq!(_creoles("{{a.jpg|b}}"), Ok(("", vec![Creole::Image("a.jpg", "b")])));
    //   assert_eq!(_creoles("|=|=a|=b|\n|0|1|2|\n|3|4|5|"), Ok(("", vec![Creole::TableHeaderCell(""), Creole::TableHeaderCell("a"), Creole::TableHeaderCell("b"), Creole::TableRowCell("0"), Creole::TableRowCell("1"), Creole::TableRowCell("2"), Creole::TableRowCell("3"), Creole::TableRowCell("4"), Creole::TableRowCell("5")])));
    //   assert_eq!(_creoles("{{{\n== [[no]]:\n//**don't** format//\n}}}"), Ok(("", vec![Creole::Text("== [[no]]:\n//**don't** format//")])));
  }

}
