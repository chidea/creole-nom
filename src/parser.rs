use {
  // std::env,
  // std::fmt::Display,
  nom::{
    IResult,
    error::{ErrorKind, },
    // bytes::{
    //   complete::{take_while},
    // },
    character::{
      // is_alphanumeric, 
      complete::{
        // char,
        not_line_ending, line_ending,
        // none_of, one_of,
        // space0, space1,
      },
    },
    combinator::{/* map, map_opt, map_res, value, verify, */ rest},
    // sequence::{delimited, preceded, tuple},
    char, tag, is_not, take_while_m_n, take, take_while1, //take_str,// re_find,
    // many0, many1_count,
  },
  crate::creole::{CreoleErr, Creole, Creoles, },
};

// named!(line, take_while!(is_not_line_end));
// named!(space, take_while!(is_space));
named!(take1, take!(1));
named!(bold<&str, &str>, delimited!(tag!("**"), is_not!("**"), tag!("**")));
named!(italic<&str, &str>, delimited!(tag!("//"), is_not!("//"), tag!("//")));
// named!(text<&str, &str, CreoleErr>, take_while1!(|c| !INLINE_MARKUP_STARTS.contains(c) ));
named!(force_linebreak<&str, &str>, tag!("\\\\"));
named!(link<&str, &str>, delimited!(tag!("[["), is_not!("]]"), tag!("]]")));
named!(image<&str, &str>, delimited!(tag!("{{"), is_not!("}}"), tag!("}}")));
named!(numberedlist<&str, &str>, terminated!(take_while1!(is_numlist), char!(' ')));
named!(bulletlist<&str, &str>, terminated!(take_while1!(is_bulletlist), char!(' ')));
named!(heading<&str, &str>, terminated!(take_while_m_n!(2, 7, is_heading), char!(' ')));
named!(horizontal<&str, &str>, tag!("----"));
named!(escape<&str, &str>, delimited!(tag!("{{{"), is_not!("}}}"), tag!("}}}")));
// fn is_markup(c: char) -> bool {
//   MARKUPS.contains(c)
// }
// fn is_not_markup(c: char) -> bool {
//   // !MARKUPS.contains(c)
//   !"*/#=[]{}-|\\".contains(c)
// }
// fn is_not_line_end(c: u8) -> bool {
//   c as char != '\n' && c as char != '\r'
// }
// fn is_not_line_end(c: char) -> bool {
//   c != '\n' && c != '\r'
// }
// fn is_asterisk(c: char) -> bool {
//   c == '*'
// }
fn is_numlist(c: char) -> bool {
  c == '#'
}
fn is_bulletlist(c: char) -> bool {
  c == '*'
}
fn is_heading(c: char) -> bool {
  c == '='
}
// fn is_space(c: char) -> bool {
//   c != ' '
// }
// fn non_markup(s:&str) -> IResult<&str, &str> {
//   nom::bytes::complete::take_while(is_not_markup)(s)
// }
// const MARKUPS :&'static str = "*/#=[]{}-|\\";
/// to check start of any markup thus stop plaintext matching,
/// it contains every first special character of markups
const INLINE_MARKUP_STARTS :&'static str = "-*/[{\\";
// const INLINE_MARKUP_ENDS :&'static str = "]}";

fn creole_line<'a>(i:&'a str) -> IResult<&'a str, Vec<Creole>> {
  debug!("line:{}", i);
  let len :usize = i.len();
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
  debug!("multi item per line left:{}", i);
  if i.len() == 0 {
    return Ok((i, vec![]))
  }
  // multi item per line
  let mut v :Vec<Creole> = vec![];
  // if cnt < len {
    let r : IResult<&'a str, Vec<Creole>> = nom::multi::many1::<&'a str, Creole, _, _>(|i:&'a str| -> IResult<&'a str, Creole> {
      let len :usize = i.len();
      if len == 0 {return Err(nom::Err::Error((i, ErrorKind::Many1)))} // done
      debug!("text left:{}:{}", len, i);
      let text = nom::bytes::complete::take_while1::<_, &str, CreoleErr>(|c| !INLINE_MARKUP_STARTS.contains(c) );

      if let Ok((i, _b)) = force_linebreak(i) {
        debug!("linebreak");
        Ok((i, Creole::Linebreak))
      } else if let Ok((i, _b)) = horizontal(i){
        debug!("horizontal line");
        return Ok((i, Creole::HorizontalLine))
      } else if let Ok((i, b)) = bold(i) {
        debug!("bold:{}", b);
        Ok((i, Creole::Bold(b.to_owned())))
      } else if let Ok((i, b)) = italic(i) {
        debug!("italic:{}", b);
        Ok((i, Creole::Italic(b.to_owned())))
      } else if let Ok((i, b)) = escape(i) {
        debug!("escape:{}", b);
        Ok((i, Creole::Text(b.to_owned())))
      } else if let Ok((i, b)) = image(i) {
        debug!("link:{}", b);
        let (name, url) = match b.chars().position(|c| c == '|'){
          Some(i) => (&b[..i], &b[i+1..]),
          None => (b, "")
        };
        Ok((i, Creole::Image(name.to_owned(), url.to_owned())))
      } else if let Ok((i, b)) = link(i) {
        debug!("image:{}", b);
        let (name, url) = match b.chars().position(|c| c == '|'){
          Some(i) => (&b[..i], &b[i+1..]),
          None => (b, "")
        };
        Ok((i, Creole::Link(name.to_owned(), url.to_owned())))
      } else if let Ok((i, b)) = text(i) {
        debug!("normal text:{}:{}:{}:{}", b.len(), b, i.len(), i);
        Ok((i, Creole::Text(b.to_owned())))
      } else {
        debug!("unfinished markup:{}:{}", i.len(), i);
        Ok((&i[1..], Creole::Text((&i[..1]).to_owned())))
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
    if let Ok((_i, vv)) = r{
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
/// (ensures parsing successes or panic)
pub fn creoles(i:&str) -> Creoles {
  try_creoles(i).unwrap().1
}

pub fn try_creoles<'a>(i:&'a str) -> IResult<&'a str, Creoles> {
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
  let (i, _r) = rest(i)?;
  Ok((i, Creoles::from(v)))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::creole::{ICreole, ICreoles};

  fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn conversion_eq_parse_tests() { init();
    let c1 = Creole::Text("ab1".to_owned());
    let c2 = ICreole::Text("ab1");
    assert_eq!(c1, c2);
    debug!("{:?}=={:?}",c1, c2);
    let v1 = Creoles::from(vec![c1]);
    let v2 = ICreoles::from(vec![c2]);
    assert_eq!(v1, v2);
    // let v3 = ICreoles::from(vec![c1]);
    // assert_eq!(v2, v3);
  //   // parse(FromStr) test
    let v4 : Creoles = "ab1".parse().unwrap();
    assert_eq!(v4, v2);
    let v5 = creoles("ab1");
    assert_eq!(v4, v5);
  }

  #[test]
  fn text_tests() { init();
    assert_eq!(creoles("ab1"), vec![ICreole::Text("ab1")]);
  }

  #[test]
  fn text_style_tests(){ init();
    assert_eq!(creoles("**a**"), vec![ICreole::Bold("a")]);
    assert_eq!(creoles("//a//"), vec![ICreole::Italic("a")]);
    assert_eq!(creoles("a**b**//c//d"), vec![ICreole::Text("a"), ICreole::Bold("b"), ICreole::Italic("c"), ICreole::Text("d")]);
  }

  #[test]
  fn linebreak_tests(){ init();
    assert_eq!(creoles("a\nb\n\nc"), vec![ICreole::Text("a"), ICreole::Text("b"), ICreole::Linebreak, ICreole::Text("c")]);
    assert_eq!(creoles("a\\\\b"), vec![ICreole::Text("a"), ICreole::Linebreak, ICreole::Text("b")]);
    assert_eq!(creoles("a\\b"), vec![ICreole::Text("a"), ICreole::Text("\\"), ICreole::Text("b")]);
  }

  #[test]
  fn list_tests() { init();
    assert_eq!(creoles("* a"), vec![ICreole::BulletList(0, "a")]);
    assert_eq!(creoles("** b"), vec![ICreole::BulletList(1, "b")]);
    assert_eq!(creoles("*** c"), vec![ICreole::BulletList(2, "c")]);

    assert_eq!(creoles("# a"), vec![ICreole::NumberedList(0, "a")]);
    assert_eq!(creoles("## b"), vec![ICreole::NumberedList(1, "b")]);
    assert_eq!(creoles("### c"), vec![ICreole::NumberedList(2, "c")]);
  }

  #[test]
  fn heading_tests(){ init();
    assert_eq!(creoles("== a"), vec![ICreole::Heading(0, "a")]);
    assert_eq!(creoles("=== b"), vec![ICreole::Heading(1, "b")]);
    assert_eq!(creoles("==== c"), vec![ICreole::Heading(2, "c")]);
  }

  #[test]
  fn link_tests(){ init();
    assert_eq!(creoles("[[a]]"), vec![ICreole::Link("a", "")]);
    assert_eq!(creoles("[[https://google.com|google]]"), vec![ICreole::Link("https://google.com", "google")]);
  }

  #[test]
  fn other_tests(){ init();
    assert_eq!(creoles("----"), vec![ICreole::HorizontalLine]);
    assert_eq!(creoles("----a"), vec![ICreole::HorizontalLine, ICreole::Text("a")]);
    assert_eq!(creoles("a\n----\nb"), vec![ICreole::Text("a"), ICreole::HorizontalLine, ICreole::Text("b")]);
    assert_eq!(creoles("{{a.jpg}}"), vec![ICreole::Image("a.jpg", "")]);
    assert_eq!(creoles("{{a.jpg|b}}"), vec![ICreole::Image("a.jpg", "b")]);
  //   assert_eq!(creoles("|=|=a|=b|\n|0|1|2|\n|3|4|5|"), vec![ICreole::TableHeaderCell(""), ICreole::TableHeaderCell("a"), ICreole::TableHeaderCell("b"), ICreole::TableRowCell("0"), ICreole::TableRowCell("1"), ICreole::TableRowCell("2"), ICreole::TableRowCell("3"), ICreole::TableRowCell("4"), ICreole::TableRowCell("5")]);
    assert_eq!(creoles("{{{== [[no]]://**don't** format//}}}"), vec![ICreole::Text("== [[no]]://**don't** format//")]);
    assert_eq!(creoles("{{{\n== [[no]]:\n//**don't** format//\n}}}"), vec![ICreole::Text("== [[no]]:\n//**don't** format//")]);
  }
}