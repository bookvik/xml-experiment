use std::fs::File;
use std::io::prelude::*;

use std::io::{BufRead, BufReader};
use std::collections::{HashSet};

extern crate encoding;

macro_rules! read_xml_by_chunks {
    ($bf:ident, $f:ident, $block:block) => {
        let mut f = BufReader::new(File::open("src/0.xml").expect("open failed"));
        let mut buf = Vec::<u8>::new();
        let mut $f = HashSet::new();

        while f.read_until(b'>', &mut buf).expect("read_until failed") != 0 {
            let s = String::from_utf8(buf).expect("from_utf8 failed");

            let $bf = s.trim();

            $block;

            if $f.contains("__stop_read__") {
                break;
            }

            buf = s.into_bytes();
            buf.clear();
        }
    }
}

macro_rules! read_tag {
    ($t:ident, $c:ident, $f:ident, $block:block) => {
        read_tag!($t, $c, $f, $block, {});
    };

    ($t:ident, $c:ident, $f:ident, $block:block, $text:block) => {
        let tag = stringify!($t);
        let cls = format!("</{}>", tag);

        if $f.contains(tag) && $c.ends_with(&cls) {
            $f.remove(tag);
            $text;
        }

        if $f.contains(tag) || $c.starts_with(&format!("<{}>", tag)) || $c.starts_with(&format!("<{} ", tag)) {
            if !$f.contains(tag) {
                $f.insert(tag);
            }

            $block;
        }
    }
}

macro_rules! read_attribute {
    ($t:ident, $c:ident, $f:ident, $block:block) => {
        let tag = stringify!($t);
        let mut $t = String::from(""); 

        $c.chars().fold(String::from(""), |mut acc, c| {
            acc.push(c);

            if $f.contains(tag) && acc.ends_with("\"") {
                $f.remove(tag);

                acc.pop();
                $t.push_str(acc.as_str());
                
                $block;
                
                acc.clear();
            }

            if !$f.contains(tag) && acc.ends_with(&format!(" {}=\"", tag)) {
                $f.insert(tag);
                acc.clear();
            }

            acc
        });
    };
}

macro_rules! get_text {
    ($t:ident, $c:ident) => {
        let beta_offset = $c.find("</").unwrap_or($c.len());
        let $t:String = String::from($c).drain(..beta_offset).collect();
    }
}

//  sugar

macro_rules! offers {
    ($c:ident, $f:ident, $block:block) => {
        read_tag!(offers, $c, $f, $block, { $f.insert("__stop_read__"); })
    }
}

macro_rules! write_xml {
    ($w:ident, $block:block) => {
        let mut $w = File::create("foo.xml")?;
        $block;
    }
}

macro_rules! write_tag {
    ($t:ident, $w:ident, $block:block) => {
        let tag = stringify!($t);

        $w.write_all(format!("<{}>", tag).as_bytes())?;
        $block;
        $w.write_all(format!("</{}>", tag).as_bytes())?;

    }
}

fn main() -> std::io::Result<()>  { 

    write_xml!(w, {
        write_tag!(yml_catalog, w, {
            write_tag!(shop, w, {
                write_tag!(offers, w, {

                    read_xml_by_chunks!(chunk, ctx,  {

                        offers!(chunk, ctx, {
                            read_tag!(offer, chunk, ctx, {
                                
                                read_tag!(price, chunk, ctx, {}, {
                                    get_text!(text, chunk);
                                    // println!("{}", text);
                                });

                                read_tag!(categoryId, chunk, ctx, {}, {
                                    get_text!(text, chunk);
                                    // println!("{}", text);
                                });
                                
                                w.write_all(chunk.as_bytes());

                            }, { 
                                w.write_all(b"</offer>");
                            });
                        });

                    });
                });
            });
        });
    });

    Ok(())
}