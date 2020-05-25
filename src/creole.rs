use std::borrow::Borrow;
use std::ops::Deref;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use nom::{
    error::{ErrorKind, ParseError},
};
use crate::parser::try_creoles;


/* Error */
#[derive(Debug, Clone, PartialEq)]
pub enum CreoleErr{
  None
}

impl ParseError<&str> for CreoleErr {
  fn from_error_kind(_input: &str, _kind: ErrorKind) -> Self {
    Self::None
  }
  fn append(_input: &str, _kind: ErrorKind, _other: Self) -> Self {
    Self::None
  }
}



#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub enum _Creole<T:Into<String>+Sized>{
pub enum _Creole<T>{
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

/// String version for wrapping up by implicit lifetime struct/enum
pub type Creole = _Creole<String>;
/// internal &str version for shorter building than Creole::Text("".to_owned())
pub type ICreole<'a> = _Creole<&'a str>;
// impl <W, S> From<_Creole<W>> for _Creole<S> where W:Deref<Target=S> {
//   fn from(v: _Creole<W>) -> Self where W:Deref<Target=S> {

//   }
// }
/*
// impl<A> From<_Creole<A>> for Creole where A:std::ops::Deref<Target = str> {
  // fn from(c: ICreole) -> Self {
  //   c.to_owned()
  // }
// }
// impl From<&ICreole<'_>> for Creole {
//   fn from(c: &ICreole) -> Self {
//     Creole::from(*c)
//   }
// }*/
impl<T, U> PartialEq<_Creole<U>> for _Creole<T> where T:Debug, U:Debug {
  fn eq(&self, other:&_Creole<U>)-> bool {
    // let s = Creole::from(self);
    // &s == other
    format!("{:?}", self) == format!("{:?}", other)
  }
}

/* Vector wrapper */
#[derive(Debug, Clone)]
pub struct _Creoles<T>(Vec<_Creole<T>>);
pub type Creoles = _Creoles<String>;
pub type ICreoles<'a> = _Creoles<&'a str>;

// impl <'a, W, S> From<Vec<_Creole<W>>> for _Creole<&S> where W:Deref<Target=S> {
//   fn from(v: Vec<_Creole<W>>) -> Self where W:Deref<Target=S> {
//     let v = {
//       let mut vv :Vec<_Creole<&S>>= vec![];
//       for i in v {
//         vv.push(i.into());
//       }
//       vv
//     };
//     _Creoles::<S>(v)
//   }
// }
// impl <'a, T:std::borrow::Borrow<str>> From<Vec<_Creole<T>>> for Creoles {
//   fn from(v: Vec<_Creole<T>>) -> Self {
//     let v = {
//       let mut vv = vec![];
//       for i in v {
//         match i {
//           _Creole::Text(t) => vv.push(Creole::Text(&t)),
//           _ => ()
//         }
//       }
//       vv
//     };
//     _Creoles::<T>(v)
//   }
// }
impl<T> From<_Creoles<T>> for Vec<_Creole<T>> {
  fn from(v: _Creoles<T>) -> Self {
    v.0
  }
}
impl<T> From<Vec<_Creole<T>>> for _Creoles<T> {
  fn from(v: Vec<_Creole<T>>) -> Self {
    _Creoles::<T>(v)
  }
}
impl<T, U> PartialEq<Vec<_Creole<U>>> for _Creoles<T> where T:Debug, U:Debug {
  fn eq(&self, other:&Vec<_Creole<U>>) -> bool {
    &self.0 == other
  }
}
impl<T, U> PartialEq<_Creoles<U>> for _Creoles<T> where T:Debug, U:Debug {
  fn eq(&self, other:&_Creoles<U>) -> bool {
    let (sl, ol) = (self.0.len(), other.0.len());
    if sl != ol { return false }
    for i in 0..sl {
      if self.0[i] != other.0[i] { return false }
    }
    true
  }
}
/*
impl <'a, T:std::borrow::Borrow<str>> From<Vec<_Creole<T>>> for _Creoles<T> {
  fn from(v: Vec<_Creole<T>>) -> Self {
    _Creoles::<T>(v)
  }
}

impl <'a, T:std::borrow::Borrow<str>> From<_Creoles<T>> for Vec<_Creole<T>> {
  fn from(w: _Creoles<T>) -> Self {
    w.0
  }
}
impl From<Creoles> for ICreoles<'_> {
  fn from(v: Creoles) -> Self {
    v.into()
  }
}
impl From<&Creoles> for ICreoles<'_> {
  fn from(v: &Creoles) -> Self {
    v.into()
  }
}
impl PartialEq<ICreoles<'_>> for Creoles {
  fn eq(&self, other:&ICreoles<'_>) -> bool {
    let s :ICreoles = self.into();
    &s == other
  }
}
impl PartialEq<Vec<ICreole<'_>>> for Creoles {
  fn eq(&self, other:&Vec<ICreole<'_>>) -> bool {
    let (sl, ol) = (self.0.len(), other.len());
    if sl != ol { return false }
    for i in 0..sl { if self.0[i] != other[i] { return false } }
    true
  }
}

*/


/* parser function connection trait */
impl std::str::FromStr for Creoles {
  type Err = (); //nom::Err<(&'str, nom::error::ErrorKind)>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match try_creoles(s){
      Ok((i, v)) => if i == "" {Ok(v.into())} else {panic!("Creole::Text is not parsed correctly")},
      Err(_e) => Err(())
    }
  }
}