use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::{Reader, Writer};

use std::io::BufReader;

use std::env;
use std::fs::File;
use std::io::Cursor;
use std::str;

extern crate percent_encoding;
extern crate crypto;

use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::str::Utf8Error;

use std::fmt;

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

/*

/// get export file 

use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = reqwest::blocking::Client::new();

    let res = client.post("http://httpbin.org/post")
    .body("the exact body that is sent")
    .send()?;

    println!("{:#?}", res);
    Ok(())
}

*/

//use json;
//use json::value::JsonValue;

fn main() {
    //let json = get_export_file(json::parse(&std::fs::read_to_string("./gq.json").unwrap()).unwrap());
    
    let args: Vec<String> = env::args().collect();
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

