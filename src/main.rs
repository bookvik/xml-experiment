use std::fs::File;
use std::time::{Instant};
use std::io::{self, BufReader, BufRead, BufWriter};
use std::io::prelude::*;


macro_rules! read_attr {
  ($t:ident, $ts:ident, $p1:ident, $v:ident, $block:block) => {
    let mut buf = Vec::<u8>::new();
    let cut = &$p1[$ts.len()..];

    for b in cut {
      buf.push(*b);

      if $t || buf == stringify!($t).as_bytes() {
        if !$t {
          $t = true;
          buf.clear();
        }

        if $t && (buf.ends_with(b"\" ") || buf.ends_with(b"\">")) {
          $t = false;
          let $v = &buf[2..buf.len()-2];
          $block;
          buf = Vec::<u8>::new();
        }
      }
    }
  }
}

macro_rules! rt { // {{{
  ($t:ident, $s:ident, $e:ident, $p1:ident, $buf:ident, $start:block, $inner:block, $end:block) => {

    if $t || $p1.starts_with($s.as_bytes()) {
      if !$t {
        $t = true;
        $start;
      }

      if $t && $buf.ends_with($e.as_bytes()) {
        $end;
        $t = false;

      } else if $t {
        $inner;
      }
    }
  }
} // }}}

macro_rules! make_lets { // {{{
  (ws, $t:ident, $s:ident, $e:ident) => {
    let mut $t = false;
    let $s = format!("<{} ", stringify!($t));
    let $e = format!("</{}>", stringify!($t));
  };

  ($t:ident, $s:ident, $e:ident) => {
    let mut $t = false;
    let $s = format!("<{}", stringify!($t));
    let $e = format!("</{}>", stringify!($t));
  }
} // }}}

macro_rules! read_xml { // {{{
  ($buf:ident, $p1:ident, $block:block) => {
    let mut f = BufReader::new(File::open("src/0.xml").expect("file not opened"));
    let mut $buf = Vec::<u8>::new();

    while f.read_until(b'>', &mut $buf).expect("read_until failed") != 0 {
      let pos = $buf.iter().position(|x| x == &60).unwrap_or(0);
      let $p1 = &$buf[pos..];
      $block;
      $buf.clear();
    }
  }
} // }}}

macro_rules! w { // {{{
  ($w:ident, $b:ident) => {
    $w.write(&$b)?;
  }
} // }}}

fn main() -> std::io::Result<()> {
  let now = Instant::now();
  let mut w = BufWriter::new(io::stdout());

  // {{{ export file settings
  let mut _cids = Vec::new();
  _cids.push(b"43");
  // }}}
 
  // {{{ lets
  make_lets!(categories, cas, cae);
  make_lets!(ws, category, cs, ce);
  make_lets!(ws, offers, ofs, ofe);
  make_lets!(offer, os, oe);
  make_lets!(url, us, ue);
  make_lets!(picture, ps, pe);
  make_lets!(categoryId, cis, cie);
  make_lets!(price, pss, pse);

  let mut id = false;
  let mut parentId = false;

  let pic_num = 1;
  // }}}

  // {{{ categories
  read_xml!(buf, p1, {

    rt!(categories, cas, cae, p1, buf, {}, {

      rt!(category, cs, ce, p1, buf, {

        read_attr!(id, cs, p1, v, {
          if _cids.iter().any(|x| x == &v) {
            println!("{:#?}", v);
          }
        });

      }, {}, {});

    }, { break; });

  }); // }}}
 
  println!("t: {}", now.elapsed().as_millis());
  println!("t: {}", now.elapsed().as_nanos());

  Ok(())
}
