use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::{Reader, Writer};

use std::io::BufReader;

use std::env;
use std::fs::File;
use std::io::{Cursor, Write};
use std::str;
use std::path::{Path, PathBuf};

extern crate percent_encoding;
extern crate crypto;


use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::str::Utf8Error;

use std::fmt;

#[macro_use]
extern crate serde;
extern crate serde_derive;

extern crate glob;
use glob::glob;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/').add(b'?').add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/*

/// web 

use actix_web::HttpResponse;

fn index_br() -> HttpResponse {
    HttpResponse::Ok().body("datadatadata\n")
}

pub fn main() {
    use actix_web::{middleware, web, App, HttpServer};
    use actix_web::http::header::ContentEncoding;

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::new(ContentEncoding::Deflate))
            .route("/", web::get().to(index_br))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}

*/

/// get export file 

use std::collections::HashMap;

#[derive(Deserialize)]
struct Shop {
    id: String
}

#[derive(Deserialize)]
struct LegacyCategory {
    ymlId: String
    ,categoryId: Option<i32>
}

#[derive(Deserialize)]
struct Category {
    id: String
}

#[derive(Deserialize)]
struct ExportFile {
    id: String
    ,filename: String
    ,isThrough: bool
    ,minPrice: Option<i32>
    ,maxPrice: Option<i32>
    ,shop: Option<Shop>
    ,legacyCategories: Vec<LegacyCategory>
    ,categories: Vec<Category>
    ,merchants: Vec<Shop>
}

struct GenericCategory {
    id: String
}

type HSS = HashMap<String, String>;

fn read_xml_from_file<F>(path: &Path, base: &[u8], v: &mut Vec<HSS>, filter: HashMap<&str, bool>, row_filter: F) 
    where F: Fn(&HSS) -> bool
{
    match Reader::from_file(path) {
        Ok(mut reader) => {
            let mut buf = Vec::new();
            reader.trim_text(true);
   
            let mut name = Vec::new();
            let mut in_row = false;
            let mut hm = HashMap::new();

            loop {
                let event = reader.read_event(&mut buf);

                match event {
                    Ok(Event::Start(ref e)) if e.name() == base => {
                        in_row = true;
                    }
                  
                    Ok(Event::Start(ref e)) if in_row => {
                        name = e.name().to_vec();
                    }

                    ,Ok(Event::Text(ref e)) if in_row => {
                        let name = str::from_utf8(name.as_slice()).unwrap().to_string();

                        if filter.is_empty() || filter.contains_key(name.as_str()) {
                            hm.insert(
                                name,
                                e.unescape_and_decode(&reader).unwrap().to_string()
                            );
                        }
                    }

                    Ok(Event::End(ref e)) if e.name() == base && in_row => {
                        in_row = false;

                        if (row_filter(&hm)) {
                            v.push(hm);

                            if v.len() > 10 {
                                break;
                            }
                        }

                        hm = HashMap::new();
                    }

                    ,Ok(Event::Eof) => break
                    ,Err(_) => println!("can not parse event on lc file")
                    ,Ok(_) => ()
                }
            }
        }

        ,Err(_) => println!("can not open lc file")
        ,Ok(_) => ()
    }
}

impl ExportFile {
    fn cats(&self, shop: &Shop) -> Vec<GenericCategory> {
        let mut v: Vec<HashMap<String,String>> = Vec::new();

        let mut filter = HashMap::new();

        filter.insert("category_id", true); 
        filter.insert("yml_id", true); 

        let cats: Vec<&str> = match self.isThrough {
            true => self.categories.iter().map(|c| c.id.as_str()).collect()
            ,false => self.legacyCategories.iter().map(|c| c.ymlId.as_str()).collect()
        };

        let key = match self.isThrough {
            true => "category_id"
            ,false => "yml_id"
        };

        read_xml_from_file(Path::new(&format!("/i4/xmls/lcs/lc_{}.xml", shop.id)), b"row", &mut v, filter, |hm| {
            match hm.get(key) {
                Some(v) => cats.iter().any(|c| c == v) 
                ,_ => false
            }
        });

        return v.iter().map(|hm| GenericCategory { id: hm.get("yml_id").unwrap().to_string() }).collect();
    }

    fn shops(&self) -> Vec<&Shop> {
        if self.isThrough {
            return self.merchants.iter().map(|s| s).collect();
        } else {
            return vec![self.shop.as_ref().unwrap()];
        }
    }
}

#[derive(Deserialize)]
struct AllExportFiles {
    allExportFiles: Vec<ExportFile>
}

#[derive(Deserialize)]
struct AllExportFilesResponse {
    data: AllExportFiles
}

fn t<F: FnOnce(&mut WCV)>(name: &[u8], w: &mut WCV, f: F) {
    w.write_event(Event::Start(BytesStart::owned(name.to_vec(), name.len())));
    f(w);
    w.write_event(Event::End(BytesEnd::borrowed(name)));
}

type WCV =  Writer<Cursor<Vec<u8>>>;

fn yml_catalog<F: FnOnce(&mut WCV)>(f: F, w: &mut WCV) {
    t(b"yml_catalog", w, f);
}

fn shop<F: FnOnce(&mut WCV)>(f: F, w: &mut WCV) {
    t(b"shop", w, f);
}

fn categories<F: FnOnce(&mut WCV)>(f: F, w: &mut WCV) {
    t(b"categories", w, f);
}

fn category<F: FnOnce(&mut WCV)>(f: F, w: &mut WCV) {
    t(b"category", w, f);
}

fn offer<F: FnOnce(&mut WCV)>(f: F, w: &mut WCV) {
    t(b"offer", w, f);
}

fn offers<F: FnOnce(&mut WCV)>(f: F, w: &mut WCV) {
    t(b"offers", w, f);
}

fn process(export_file: &ExportFile, mut w: WCV) -> Result<(), Box<dyn std::error::Error>> {

    let cats: Vec<GenericCategory> = export_file
        .shops()
        .iter()
        .map( |shop| export_file.cats(shop) )
        .into_iter()
        .flatten()
        .collect();

    let feeds: Vec<PathBuf> = export_file
        .shops().iter()
        .map( |shop| glob(&format!("/i4/slon-i4-downloader/xmls/{}/*.xml", shop.id)) )
        .filter(Result::is_ok)
        .flat_map(Result::unwrap)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect();

    let mut v: Vec<HashMap<String, String>> = Vec::new();

    feeds.iter().for_each( |pb| {
        read_xml_from_file(pb.as_path(), b"category", &mut v, HashMap::new(), |hm| {
            println!("{:#?}", hm);
            return true;
        });
    });
    
    let mut v: Vec<HashMap<String, String>> = Vec::new();

    feeds.iter().for_each( |pb| {
        read_xml_from_file(pb.as_path(), b"offer", &mut v, HashMap::new(), |hm| {
            println!("{:#?}", hm);
            return true;
        });
    });

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut map = HashMap::new();

    map.insert("query", "{ allExportFiles(filter: { filename: \"dc5f952d28ef97e02f553e7f1b0b5cbe281eac2c\"  }, perPage: 1, page: 0)  { id, filename, isThrough, partnerTrackCode, minPrice, maxPrice, state, shop { id }, format, legacyCategories { ymlId, categoryId }, parkedDomain { id, name }, categories { id }, user { apiToken }, merchants { id }}}");

    let client = reqwest::blocking::Client::new();

    let res = client
        .post("https://federation.gdeslon.ru/graphql")
        .bearer_auth("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1aWQiOiItMSJ9.naTEm5Y4Y6nGLc4t5EoLuwJPZOoXMRdw_uH0lXwGr2o")
        .json(&map)
        .send()?;


    let rsp: AllExportFilesResponse = res.json()?;

    process(
        &rsp.data.allExportFiles[0],
        Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2)
    )?;

    return Ok(());
}

/*


fn main() {
    let mid = "42071";

    let mut reader = Reader::from_file(&args[1]).unwrap();
    let mut buf = Vec::new();
    reader.trim_text(true);

    //let f = File::create("./foo.txt").unwrap();
    //let mut writer = Writer::new_with_indent(f, b' ', 2);

    let mut mem_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // export file settings

    let mut image_text = String::new();
    let mut url_text = String::new();
    let mut category_id_text = String::new();

    let mut in_offer = false;
    let mut in_picture = false;
    let mut in_url = false;
    let mut in_price = false;
    let mut in_category_id = false;
    let mut picture_num = 0;

    let now = std::time::Instant::now();
    let mut attempts = 0;

    let mut next = { () };

    loop {
        let event = reader.read_event(&mut buf);

        match event {
            Ok(event) => {
                match event {
                    // offer (modify attributes)

                    Event::Start(ref e) if e.name() == b"offer" => {
                        in_offer = true;
                        let mut elem = e.clone();

                        elem.clear_attributes();

                        let mut article = e.attributes().filter( |attr| attr.as_ref().unwrap().key == b"id" ).next().unwrap().unwrap().value;

                        // id

                        let mut hasher = Sha1::new();
                        hasher.input_str(&format!("{}|{}", mid, str::from_utf8(article.as_ref()).unwrap()));
                        elem.push_attribute(("id", u64::from_str_radix(&hasher.result_str()[0..16], 16).unwrap().to_string().as_str()));
                        
                        // article

                        elem.push_attribute(("article".as_bytes(), article.to_mut().as_slice()));

                        // another

                        elem.extend_attributes(e.attributes().filter(|attr| attr.as_ref().unwrap().key != b"id" ).map(|attr| attr.unwrap()));

                        elem.push_attribute(("merchant_id", mid));
                        elem.push_attribute(("gs_category_id", "1"));
                        elem.push_attribute(("gs_product_key", "1"));

                        mem_writer.write_event(Event::Start(elem)).unwrap();
                    }

                    Event::End(ref e) if e.name() == b"offer" => {
                        in_offer = false;
                        mem_writer.write_event(event).unwrap();

                        println!("{}", str::from_utf8(mem_writer.into_inner().into_inner().as_ref()).unwrap());

                        //attempts += 1;
                        //if attempts > 10000 {
                          //  println!("{}", str::from_utf8(mem_writer.into_inner().into_inner().as_ref()).unwrap());
                            break;
                        //}
                    }

                    // picture (append thumbnail imgng anb original_url)

                    Event::Start(ref e) if e.name() == b"picture" => {
                        if in_offer {
                            in_picture = true;

                            // picture
                            
                            let mut elem = e.clone();
                            elem.push_attribute(("id", picture_num.to_string().as_ref()));
                            mem_writer.write_event(Event::Start(elem)).unwrap();
                        }
                    }
                    
                    Event::End(ref e) if e.name() == b"picture" => {
                        if in_offer {
                            mem_writer.write_event(event).unwrap();

                            // thumnail

                            let mut thum = BytesStart::owned(b"thumbnail".to_vec(), "thumbnail".len());
                            thum.push_attribute(("id", picture_num.to_string().as_ref()));
                            mem_writer.write_event(Event::Start(thum)).unwrap();
                            mem_writer.write_event(Event::CData(BytesText::from_plain_str(&image_text))).unwrap();
                            mem_writer.write_event(Event::End(BytesEnd::borrowed(b"thumbnail"))).unwrap();
                            
                            // original_picture

                            let mut org_pic = BytesStart::owned(b"original_picture".to_vec(), "original_picture".len());
                            org_pic.push_attribute(("id", picture_num.to_string().as_ref()));
                            mem_writer.write_event(Event::Start(org_pic)).unwrap();
                            mem_writer.write_event(Event::CData(BytesText::from_plain_str(&image_text))).unwrap();
                            mem_writer.write_event(Event::End(BytesEnd::borrowed(b"original_picture"))).unwrap();
                            
                            in_picture = false;
                            picture_num += 1;
                        }
                    }

                    // url  (convert to deeplink)
                    
                    Event::Start(ref e) if e.name() == b"url" => {
                        if in_offer {
                            in_url = true;
                        }
                    }

                    Event::Start(ref e) if e.name() == b"categoryId" => {
                        if in_offer {
                            in_category_id = true;
                            mem_writer.write_event(event).unwrap();
                        }
                    }

                    Event::End(ref e) if e.name() == b"categoryId" => {
                        if in_offer && in_category_id {
                            in_category_id = false;
                            mem_writer.write_event(event).unwrap();
                        }
                    }

                    Event::End(ref e) if e.name() == b"url" => {
                        if in_offer && in_url {

                            let tag_name = b"url";

                            let mut thum = BytesStart::owned(tag_name.to_vec(), tag_name.len());
                            
                            mem_writer.write_event(Event::Start(thum)).unwrap();
                            
                            mem_writer.write_event(Event::CData(BytesText::from_plain_str(
                                        &format!("https://f.gdeslon.ru/{}?mid={}&goto={}", "test", mid, utf8_percent_encode(&url_text, FRAGMENT).to_string())
                            ))).unwrap();

                            mem_writer.write_event(Event::End(BytesEnd::borrowed(tag_name))).unwrap();

                            let tag_name = b"destination-url-do-not-send-traffic";

                            let mut thum = BytesStart::owned(tag_name.to_vec(), tag_name.len());
                            
                            mem_writer.write_event(Event::Start(thum)).unwrap();
                            mem_writer.write_event(Event::CData(BytesText::from_plain_str(&url_text))).unwrap();
                            mem_writer.write_event(Event::End(BytesEnd::borrowed(tag_name))).unwrap();

                            in_url = false;
                        }
                    }

                    // another not changable elements
                    
                    // text processing

                    Event::Text(ref e) | Event::CData(ref e) => {
                        if in_offer {
                            if in_picture {
                                image_text = e.unescape_and_decode(&reader).unwrap().to_string();
                                mem_writer.write_event(Event::CData(e.clone())).unwrap();
                            } else if in_url {
                                url_text = e.unescape_and_decode(&reader).unwrap().to_string();
                            } else if in_category_id {
                                category_id_text = e.unescape_and_decode(&reader).unwrap().to_string();
                                mem_writer.write_event(Event::CData(e.clone())).unwrap();
                            } else {
                                mem_writer.write_event(Event::CData(e.clone())).unwrap();
                            }
                        }
                    }

                    Event::Start(ref e) if e.name() != b"offer" => {
                        if in_offer {
                            mem_writer.write_event(event).unwrap();
                        }
                    }

                    Event::End(ref e) if e.name() != b"offer" => {
                        if in_offer {
                            mem_writer.write_event(event).unwrap();
                        }
                    }
                    
                    Event::Empty(_) => {
                        if in_offer {
                            mem_writer.write_event(event).unwrap();
                        }
                    }


                    Event::Eof => break, // exits the loop when reaching end of file

                    _ => (), // There are several other `Event`s we do not consider here
                }
            }

            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        buf.clear();
    }

    let elapsed = now.elapsed().as_nanos();
    println!("Elapsed {} ns", elapsed);
}

*/

