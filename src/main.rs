use std::io::{Cursor, BufReader};
use std::fs::File;

use quick_xml::{Reader, Writer};
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};

struct XMLAttr {
  key: String
  ,value: String
}

impl XMLAttr {
  fn new(k: &str, v: &str) -> Self {
    Self {
      key: s(k)
      ,value: s(v)
    }
  }
}

struct XMLNode {
  name: String
  ,attrs: Vec<XMLAttr>
  ,value: String
  ,nodes: Vec<XMLNode>
}

impl XMLNode {
  fn new(name: &str) -> Self {
    Self {
      name: s(name)
      ,attrs: vec![]
      ,value: s("")
      ,nodes: vec![]
    }
  }
}

type WCVU8 = Writer<Cursor<Vec<u8>>>;

struct XMLWriter {
  writer: WCVU8
}

impl XMLWriter {
  fn new(w: WCVU8) -> Self{
    Self {
      writer: w
    }
  }

  fn into_inner(self) -> WCVU8 {
    self.writer
  }

  fn write(&mut self, x: &XMLNode) {
    let xname = x.name.as_bytes();

    let mut elem = BytesStart::owned(xname.to_vec(), xname.len());

    if !x.attrs.is_empty() {
      x.attrs.iter().for_each( |attr| {
        elem.push_attribute((attr.key.as_str(), attr.value.as_str()));
      });
    }

    self.writer.write_event(Event::Start(elem));

    if !x.nodes.is_empty() {
      x.nodes.iter().for_each( |xnode| self.write(xnode) );
    } else if !x.value.is_empty() {
      self.writer.write_event(Event::CData(BytesText::from_plain_str(&x.value))).unwrap();
    }

    self.writer.write_event(Event::End(BytesEnd::borrowed(xname)));
  }
}


struct XMLReader {
  reader: Reader<BufReader<File>>
  ,tag: String
  ,name: String
  ,buf: Vec<u8>
}

impl XMLReader {
  fn from_file(f: &str, tag: &str, name: &str) -> Self {
    let mut reader = Reader::from_file(f).unwrap();
    reader.trim_text(true);

    let mut buf = Vec::new();

    Self {
      reader
      ,tag: s(tag)
      ,name: s(name)
      ,buf
    }
  }

  fn fltr(xnode: &mut XMLNode) -> bool {
      match xnode.name.as_str() {
        "yml_id" | "category_id" => true
        ,_ => false
      }
  }
}

impl Iterator for XMLReader {
  type Item = XMLNode;


  fn next(&mut self) -> Option<Self::Item> {
    let mut xnodes: Vec<XMLNode> = vec![];

    loop {
        self.buf.clear();
        let event = self.reader.read_event(&mut self.buf);

        match event {
          Ok(Event::Start(ref e)) if e.name() == self.tag.as_bytes() => {
            let mut xnode = XMLNode::new(&self.name);
            xnodes.push(xnode);
          }
          
          ,Ok(Event::Start(ref e)) if !xnodes.is_empty() =>  {
            let name = std::str::from_utf8(e.name()).unwrap();
            let mut xnode = XMLNode::new(name);
            xnodes.push(xnode) 
          }
          
          ,Ok(Event::Text(ref e)) if !xnodes.is_empty() =>  {
            let value = e.unescape_and_decode(&self.reader).unwrap().to_string();

            match xnodes.last_mut() {
              Some(v) => v.value = value 
              ,_ => () 
            }
          }

          ,Ok(Event::End(ref e)) =>  {
            let popped = xnodes.pop();

            if xnodes.is_empty() {
              return popped;
            } else {
                match xnodes.last_mut() {
                    Some(v) => {
                        let mut pun = popped.unwrap();

                        if XMLReader::fltr(&mut pun) {
                            v.nodes.push(pun)
                        }
                    }
                    _ => panic!("wrong branch")
                }
            }
          }

          ,Ok(Event::Eof) => return None

          ,_ => ()
        }
    }
  }
}

struct Shop {
	id: String
}

struct Category {
    id: String
}

struct LegacyCategory {
    categoryId: Option<i32>
    ,ymlId: String
}

struct ExportFile {
	filename: String
	,isThrough: bool
	,shop: Option<Shop>
	,merhants: Vec<Shop>
    ,categories: Vec<Category>
    ,legacyCategories: Vec<LegacyCategory>
}

/// helper

fn s(s: &str) -> String {
    s.to_string()
}

fn main() {
  let ef  = ExportFile {
    filename: s("28e98107ae3b1d68622fcd4a76e12ace307eb09c")
    ,isThrough: true
    ,merhants: vec![Shop { id: s("1") }]
    ,shop: Some(Shop { id: s("1") })
    ,categories: vec![Category { id: s("1") }]
    ,legacyCategories: vec![LegacyCategory { categoryId: Some(1), ymlId: s("123") }]
  };

  if ef.isThrough {
        let cat_ids: Vec<&str> = ef.categories.iter().map(|c| c.id.as_str()).collect();

		for shop in ef.merhants {
            let mut xreader = XMLReader::from_file("/i4/xmls/lcs/lc_42071.xml", "row", "category");
		}

  } else {
      let yml_ids: Vec<&str> = ef.legacyCategories.iter().map(|c| c.ymlId.as_str()).collect();
  }

  let mut w = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
  let mut xwriter = XMLWriter::new(w);

  let mut yml_catalog = XMLNode::new("yml_catalog");
  let mut shop = XMLNode::new("shop");

  let mut categories = XMLNode::new("categories");
  let mut offers = XMLNode::new("offers");

  let mut xreader = XMLReader::from_file("/i4/xmls/lcs/lc_42071.xml", "row", "category");
  
  for node in xreader {
    categories.nodes.push(node);
    break;
  }

  shop.nodes.push(categories);
  shop.nodes.push(offers);
  yml_catalog.nodes.push(shop);

  xwriter.write(&yml_catalog);

  println!("{}", std::str::from_utf8(xwriter.into_inner().into_inner().into_inner().as_ref()).unwrap().to_string());
}
