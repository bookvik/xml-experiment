use std::fs::File;
use std::time::{Instant};
use std::io::{self, BufReader, BufRead, BufWriter};
use std::io::prelude::*;
use std::collections::{HashSet};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

macro_rules! read_attr { // {{{
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
} // }}}

macro_rules! rt { // {{{
  ($t:ident, $s:ident, $e:ident, $p1:ident, $buf:ident, $start:block, $inner:block, $end:block) => {

    if $t || $p1.starts_with($s.as_bytes()) {
      if !$t {
        $t = true;
        $start;
      }

      if $t && $buf.ends_with($e.as_bytes()) {
        $t = false;
        $end;

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
  ($source:ident, $buf:ident, $p1:ident, $block:block) => {
    let mut f = BufReader::new($source);
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

macro_rules! ps { // {{{
  ($x:ident) => {
    println!("{:#?}", String::from_utf8($x.to_vec()));
  }
} // }}}

macro_rules! read_text { // {{{
  ($t:ident, $es:ident, $buf:ident) => {
    let $t = &$buf[0..$buf.len() - $es.len()]; 
  }
} // }}}

macro_rules! gql { // {{{
  ($res:ident, $body:expr, $block:block) => {
    let client = reqwest::blocking::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // TO-DO rewrite on federation plz
    let $res = client.post("https://www.gdeslon.ru/adminka/graphql.xml")
                    .bearer_auth(std::env::var("BEARER_TOKEN").expect("BEARER_TOKEN"))
                    .headers(headers)
                    .body($body)
                    .send()?;

    $block;
  }
} // }}}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut w = BufWriter::new(io::stdout());
  let mut tmpw = Vec::<u8>::new();
 
  // {{{ lets 
  make_lets!(ws, allExportFile, als, ale);
  make_lets!(ws, filename, fs, fe);
  make_lets!(ws, isThrough, its, ite);

  make_lets!(categories, cas, cae);
  make_lets!(ws, category, cs, ce);
  make_lets!(offers, ofs, ofe);
  make_lets!(ws, offer, os, oe);
  make_lets!(url, us, ue);
  make_lets!(picture, ps, pe);
  make_lets!(categoryId, cis, cie);
  make_lets!(price, pss, pse);
  make_lets!(id, ids, ide);
  make_lets!(ymlId, ys, ye);

  let mut id = false;
  let mut pic_num = 1;

  // }}}
  
  // {{{ export file settings

  let body = [r#"{"query": "{ allExportFiles(page:0, perPage: 1, filter: { filename: \""#, std::env::args().nth(1).unwrap().as_str(), r#"\" }) { filename, shop { id }, merchants { id }, isThrough, categories { legacyCategories { ymlId }}, legacyCategories { ymlId }, parkedDomain { name }, partnerTrackCode }}"}"#].concat();

  let mut mids = HashSet::<Vec<u8>>::new();
  let mut ymls = HashSet::<Vec<u8>>::new();

  gql!(res, body, {
    read_xml!(res, buf, p1, {

      rt!(id, ids, ide, p1, buf, {}, {}, {
        read_text!(_id, ide, buf);
        mids.insert(_id.to_vec());
      });

      rt!(ymlId, ys, ye, p1, buf, {}, {}, {
        read_text!(_ymlId, ye, buf);
        ymls.insert(_ymlId.to_vec());
      });

    });
  });

  // }}}

  let now = Instant::now();

  for mid in mids { 
  }

  // {{{ categories

  let f = File::open("src/0.xml").expect("file not opened");

  read_xml!(f, buf, p1, {

    rt!(categories, cas, cae, p1, buf, {}, {

      rt!(category, cs, ce, p1, buf, {

        read_attr!(id, cs, p1, v, {
          category = ymls.iter().any(|x| x == &v);
        });

      }, {}, {});

    }, { break; });

  }); // }}}

  // {{{ offers

  let f = File::open("src/0.xml").expect("file not opened");

  read_xml!(f, buf, p1, { 

    rt!(offers, ofs, ofe, p1, buf, {}, {
      
      rt!(offer, os, oe, p1, buf, {}, {

        rt!(url, us, ue, p1, buf, {}, {}, {
          //read_text!(text, ue, buf);
        });
      
        rt!(categoryId, cis, cie, p1, buf, {}, {}, {
          //read_text!(text, cie, buf);
          //offer = _cids.iter().any(|x| x == &text);
        });

        rt!(price, pss, pse, p1, buf, {}, {}, {
          //read_text!(text, pse, buf);

          //if min_price.is_some() {
          //  let i = text.split(|x| x == &'.');
          //  offer = text >= min_price.unwrap();
          //} 

          //if max_price.is_some() {
          //  offer = text <= max_price.unwrap();
          //}
        });
        
        rt!(picture, ps, pe, p1, buf, {}, {}, {
          //read_text!(text, pe, buf);
        });

      }, { });

    }, { break; });

  }); // }}}
 
  println!("t: {}", now.elapsed().as_millis());
  println!("t: {}", now.elapsed().as_nanos());

  Ok(())
}
