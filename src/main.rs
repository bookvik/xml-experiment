use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::events::attributes::{Attribute};
use quick_xml::{Reader, Writer};
use std::ffi::OsStr;
use std::io::{Cursor, BufReader};
use zip::ZipWriter;
use std::fs::File;

extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

extern crate percent_encoding;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::str::Utf8Error;

extern crate glob;
use glob::glob;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/').add(b'?').add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

fn s(s: &str) -> String {
    s.to_string()
}

fn us(s: &[u8]) -> &str {
    std::str::from_utf8(&s).unwrap_or("")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yml_ids = vec!["740", "745"];

    let min_price = None; //Some(1000.0);
    let max_price = None; //Some(2000.0);

    let mids = vec!["80549"];
    let api_token = "8ee42be72a";

    let parked_domain = None;
    let partner_track_code = None;

    let filename = "dc5f952d28ef97e02f553e7f1b0b5cbe281eac2c.xml.zip";

    let write = File::create(filename)?;
    let mut zip = ZipWriter::new(write);

    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("0.xml", options)?;
    let mut writer = Writer::new_with_indent(zip, b' ', 2);
    
    let tag  = b"yml_catalog";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));
    
    let tag  = b"shop";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));

    // {{{ categories
    for mid in &mids {

    let tag  = b"categories";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));

    let paths = glob(&format!("/i4/slon-i4-downloader/xmls/{}/*.xml", mid))?;
    for path in paths {
        let mut reader = Reader::from_file(path?)?;
        reader.trim_text(true);
        
        let mut buf = Vec::new();

        let mut in_category = false;
        let mut category_text = String::new();

        let mut offer: Option<BytesStart> = None;

        loop {
            let event = reader.read_event(&mut buf)?;

            match event {

                // category

                Event::Start(ref e) if !in_category && e.name() == b"category" => {
                    for attr in e.attributes() {
                        let a = attr?;

                        if a.key == b"id" {
                            in_category = yml_ids.iter().any(|yid| yid == &us(a.value.as_ref()));
                            break;
                        }
                    }

                    if in_category {
                        writer.write_event(event)?;
                    }
                }
                
                ,Event::Text(ref e) if in_category => {
                    writer.write_event(event)?;
                }

                ,Event::End(ref e) if in_category && e.name() == b"category" => {
                    in_category = false;
                    writer.write_event(event)?;
                }
                
                ,Event::Eof => break
            
                ,_ => ()
            }
        }
    }

    writer.write_event(Event::End(BytesEnd::borrowed(tag)));

    }

    // }}}
    
    // {{{ offers

    for mid in &mids {

   
    let tag  = b"offers";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));

    let paths = glob(&format!("/i4/slon-i4-downloader/xmls/{}/*.xml", mid))?;

    for path in paths {
        let mut reader = Reader::from_file(path?)?;
        reader.trim_text(true);
        
        let mut buf = Vec::new();

        let mut in_offer = false;
        let mut in_url = false;
        let mut in_picture = false;
        let mut in_price = false;
        let mut in_categoryId = false;

        let mut url_text = String::new();
        let mut picture_text = String::new();
        let mut price_text = String::new();
        let mut categoryId_text = String::new();
        let mut article_text = String::new();

        let mut offer_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        loop {
            let event = reader.read_event(&mut buf)?;

            match event {
                // offer
                
                Event::Start(ref e) if !in_offer && e.name() == b"offer" => {
                    in_offer = true;

                    let mut elem = e.clone();

                    elem.clear_attributes();

                    // article

                    let mut article = e.attributes().filter( |attr| attr.as_ref().unwrap().key == b"id" ).next().unwrap().unwrap().value;

                    article_text = s(us(article.as_ref()));
                    elem.push_attribute(("article".as_bytes(), article.as_ref()));

                    // id
                    
                    let mut hasher = Sha1::new();
                    hasher.input_str(&format!("{}|{}", mid, us(article.as_ref())));
                    elem.push_attribute(("id", u64::from_str_radix(&hasher.result_str()[0..16], 16)?.to_string().as_str()));

                    // another

                    elem.extend_attributes(e.attributes().filter(|attr| attr.as_ref().unwrap().key != b"id" ).map(|attr| attr.unwrap()));

                    elem.push_attribute(("merchant_id", mid.to_string().as_ref()));
                    elem.push_attribute(("gs_category_id", "1"));
                    elem.push_attribute(("gs_product_key", "1"));

                    offer_writer.write_event(Event::Start(elem))?;
                }

                ,Event::End(ref e) if in_offer && e.name() == b"offer" => {
                    in_offer = false;

                    offer_writer.write_event(event)?;
                    writer.write(offer_writer.into_inner().into_inner().as_ref());

                    offer_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
                }
                
                // offer categoryId
                
                ,Event::Start(ref e) if in_offer && e.name() == b"categoryId" => {
                    in_categoryId = true;
                    offer_writer.write_event(event)?;
                }

                ,Event::Text(ref e) if in_offer && in_categoryId => {
                    categoryId_text = e.unescape_and_decode(&reader)?;
                    offer_writer.write_event(event)?;
                }
                
                ,Event::End(ref e) if in_offer && in_categoryId && e.name() == b"categoryId" => {
                    in_categoryId = false;
                    in_offer = yml_ids.iter().any(|yid| yid == &categoryId_text );

                    if in_offer {
                        offer_writer.write_event(event)?;
                    } else {
                        offer_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
                    }
                }

                // offer price
                
                ,Event::Start(ref e) if in_offer && e.name() == b"price" => {
                    in_price = true;
                    offer_writer.write_event(event)?;
                }

                ,Event::Text(ref e) if in_offer && in_price => {
                    price_text = e.unescape_and_decode(&reader)?;
                    offer_writer.write_event(event)?;
                }
                
                ,Event::End(ref e) if in_offer && in_price && e.name() == b"price" => {
                    in_price = false;

                    let price_text: f32 = price_text.parse()?;

                    if min_price.is_some() {
                        in_offer = price_text >= min_price.unwrap();
                    } 

                    if in_offer && max_price.is_some() {
                        in_offer = price_text <= max_price.unwrap();
                    }

                    if in_offer {
                        offer_writer.write_event(event)?;
                    } else {
                        offer_writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
                    }
                }
                
                // offer url 

                ,Event::Start(ref e) if in_offer && e.name() == b"url" => {
                    in_url = true;
                }

                ,Event::Text(ref e) if in_offer && in_url => {
                    url_text = e.unescape_and_decode(&reader)?.to_string();
                }
                
                ,Event::End(ref e) if in_offer && in_url && e.name() == b"url" => {
                    let tag = b"url";

                    let mut sid  = String::new();

                    if partner_track_code.is_some() {
                        sid = format!("&sub_id={}", partner_track_code.unwrap_or(""));
                    } else {
                        sid = s("");
                    }

                    let deep_link = format!("{}/cf/{}?mid={}&goto={}{}", 
                                            parked_domain.unwrap_or("https://f.gdeslon.ru"), 
                                            api_token, 
                                            mid, 
                                            utf8_percent_encode(&url_text, FRAGMENT).to_string(),
                                            sid
                                           );

                    offer_writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));
                    offer_writer.write_event(Event::CData(BytesText::from_plain_str(&deep_link)))?;
                    offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)));

                    let tag = b"destination-url-do-not-send-traffic";

                    offer_writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));
                    offer_writer.write_event(Event::CData(BytesText::from_plain_str(&url_text)))?;
                    offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)));

                    in_url = false;
                }
                
                // offer picture
                
                ,Event::Start(ref e) if in_offer && e.name() == b"picture" => {
                    in_picture = true;
                }

                ,Event::Text(ref e) if in_offer && in_picture => {
                    picture_text = e.unescape_and_decode(&reader)?.to_string();
                }
                
                ,Event::End(ref e) if in_offer && in_picture && e.name() == b"picture" => {
                    in_picture = false;

                    let tag = b"picture";
                    let im_no = "0";

                    let mut hasher = Sha1::new();
                    hasher.input_str(&picture_text);

                    let small_hash = hasher.result_str();

                    offer_writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));
                    offer_writer.write_event(Event::CData(BytesText::from_plain_str(&format!("https://imgng.gdeslon.ru/mid/{}/imno/{}/cid/{}/hash/{}/{}.jpg", mid, im_no, article_text, small_hash, "big"))))?;
                    offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)));

                    let tag = b"thumbnail";
                    let small_hash = hasher.result_str();

                    offer_writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));
                    offer_writer.write_event(Event::CData(BytesText::from_plain_str(&format!("https://imgng.gdeslon.ru/mid/{}/imno/{}/cid/{}/hash/{}/{}.jpg", mid, im_no, article_text, small_hash, "small"))))?;
                    offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)));
                    
                    let tag = b"original_picture";

                    offer_writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())));
                    offer_writer.write_event(Event::CData(BytesText::from_plain_str(&picture_text)))?;
                    offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)));
                }

                ,Event::Eof => break
            
                ,Event::Start(_) if in_offer => {
                    offer_writer.write_event(event)?;
                }
                
                ,Event::Text(ref e) if in_offer => {
                    offer_writer.write_event(Event::CData(e.clone()))?;
                }
                
                ,Event::End(_) if in_offer => {
                    offer_writer.write_event(event)?;
                }

                ,_ if in_offer => {
                    offer_writer.write_event(event)?;
                }

                ,_ => ()
            }

            buf.clear();
        }
    }

    writer.write_event(Event::End(BytesEnd::borrowed(tag)));

    }
   

    // }}}

    let tag = b"shop";
    writer.write_event(Event::End(BytesEnd::borrowed(tag)));
    
    let tag = b"yml_catalog";
    writer.write_event(Event::End(BytesEnd::borrowed(tag)));

    //println!("{}", s(us(writer.into_inner().into_inner().as_ref())));
    
    Ok(())
}
