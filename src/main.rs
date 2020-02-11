use std::fs::File;
use std::time::{Instant};
use std::io::{self, BufReader, BufRead, BufWriter};
use std::io::prelude::*;

macro_rules! read_attr { // {{{
  ($t:ident, $s:ident, $p1:ident) => {
    // cut tag start identificator
    let cut = &$p1[$s.len()..$p1.len()-1];

    println!("{:#?}", cut);
    break;
  }
} // }}}

macro_rules! rt { // {{{
  ($t:ident, $s:ident, $e:ident, $p1:ident, $buf:ident, $start:block, $end:block) => {
    if $t || $p1.starts_with($s.as_bytes()) {
      if !$t {
        $t = true;
      }

      if $t {
        $start;
      }

      if $t && $buf.ends_with($e.as_bytes()) {
        $end;
        $t = false;
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
      let pos = $buf.binary_search(&60).unwrap_or(0);
      let $p1 = &$buf[pos..];

      $block;

      $buf.clear();
    }
  }
} // }}}

macro_rules! us { // {{{
  ($t:ident) => {
    println!("{}", String::from_utf8($t).unwrap());
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
 
  // {{{ lets
  make_lets!(categories, cas, cae);
  make_lets!(ws, category, cs, ce);
  make_lets!(ws, offers, ofs, ofe);
  make_lets!(offer, os, oe);
  make_lets!(url, us, ue);
  make_lets!(picture, ps, pe);
  make_lets!(categoryId, cis, cie);
  make_lets!(price, pss, pse);
  let pic_num = 1;
  // }}}

  // {{{ categories
  read_xml!(buf, p1, {

    rt!(categories, cas, cae, p1, buf, {

      rt!(category, cs, ce, p1, buf, {
        read_attr!(id, cs, p1);

      }, {});

    }, { break; });

  }); // }}}
 
  // {{{ offers 
  read_xml!(buf, p1, {

    rt!(offers, ofs, ofe, p1, buf, {

      rt!(offer, os, oe, p1, buf, {
        rt!(url, us, ue, p1, buf, {}, {
        });

        rt!(price, pss, pse, p1, buf, {}, {
        });
        
        rt!(categoryId, cis, cie, p1, buf, {}, {
        });

        rt!(picture, ps, pe, p1, buf, {}, {
        });

      }, {});

    }, { break; });

  }); // }}}

  println!("t: {}", now.elapsed().as_millis());
  println!("t: {}", now.elapsed().as_nanos());

  Ok(())
}
