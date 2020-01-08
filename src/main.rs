use std::env;
use std::io::{Cursor};
use std::fs::File;
use std::collections::HashMap;

use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::{Reader, Writer};

use zip::ZipWriter;

extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

extern crate percent_encoding;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

extern crate glob;
use glob::glob;

use serde::Deserialize;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/').add(b'?').add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

fn s(s: &str) -> String {
    s.to_string()
}

fn us(s: &[u8]) -> &str {
    std::str::from_utf8(&s).unwrap_or("")
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GraphqlResponseInner {
   all_export_files: Vec<ExportFile> 
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GraphqlResponse {
    data: GraphqlResponseInner
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LegacyCategory {
    category_id: Option<i32>
    ,merchant_id: i32
    ,yml_id: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Category {
    legacy_categories: Vec<LegacyCategory>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ParkedDomain {
    name: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Shop {
    id: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct User {
    api_token: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ExportFile {
    filename: String
    ,is_through: bool
    ,categories: Vec<Category>
    ,legacy_categories: Vec<LegacyCategory>
    ,min_price: Option<f32>
    ,max_price: Option<f32>
    ,parked_domain: Option<ParkedDomain>
    ,shop: Option<Shop>
    ,merchants: Vec<Shop>
    ,user: User
    ,partner_track_code: Option<String>
}

fn main2() -> Result<(), Box<dyn std::error::Error>> {
    let mut map = HashMap::new();

    map.insert("query", r#"{ 
        allExportFiles(page: 0, perPage: 1) { 
            isThrough,
            filename, 
            minPrice, 
            maxPrice,
            legacyCategories { categoryId, ymlId, merchantId },
            categories { legacyCategories { categoryId, ymlId, merchantId } },
            shop { id },
            merchants { id },
            parkedDomain { name },
            user { apiToken },
            partnerTrackCode
        }
    }"#);

    let client = reqwest::blocking::Client::new();

    let res = client
        .post("https://federation.gdeslon.ru/graphql")
        .bearer_auth("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1aWQiOiItMSJ9.naTEm5Y4Y6nGLc4t5EoLuwJPZOoXMRdw_uH0lXwGr2o")
        .json(&map)
        .send()?;

    //println!("{}", res.text()?);

    let json: GraphqlResponse =  res.json()?;

    let export_file = json.data.all_export_files.first().unwrap();

    let min_price = export_file.min_price;
    let max_price = export_file.max_price;

    let mids: Vec<String> = match export_file.is_through {
        true => export_file.merchants.iter().map ( |m| s(m.id.as_str()) ).collect() 
        ,false => vec![export_file.shop.as_ref().unwrap().id.to_string()]
    };

    let api_token = export_file.user.api_token.get(0..10).unwrap();

    let filename = export_file.filename.as_str();

    let write = File::create(format!("{}.xml.zip", filename))?;
    let mut zip = ZipWriter::new(write);

    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(format!("{}.xml", filename), options)?;
    let mut writer = Writer::new_with_indent(zip, b' ', 2);
    
    let tag  = b"yml_catalog";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())))?;
    
    let tag  = b"shop";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())))?;

    println!("{:#?}", export_file);

    println!("Start of categories");

    let tag  = b"categories";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())))?;

    // {{{ categories
    for mid in &mids {
        println!("Start of {}", mid);

        let yml_ids: Vec<&str> = match export_file.is_through {
            true => export_file.categories.iter().flat_map(|c| &c.legacy_categories ).filter(|lc| lc.merchant_id.to_string().as_str()  == mid ).map(|lc| lc.yml_id.as_str() ).collect()
            ,false => export_file.legacy_categories.iter().filter(|lc| lc.merchant_id.to_string().as_str() == mid ).map( |lc| lc.yml_id.as_str() ).collect()
        };

        let paths = glob(&format!("/i4/slon-i4-downloader/xmls/{}/*.xml", mid))?;

        for path in paths {
            println!("start of part of {}", mid);

            let mut reader = Reader::from_file(path?)?;
            reader.trim_text(true);
            
            
            let mut buf = Vec::new();

            let mut in_category = false;
            let mut category_text = String::new();

            let mut offer: Option<BytesStart> = None;

            loop {
                let event = reader.read_event(&mut buf)?;

                match event {

                    Event::Decl(ref e) =>  {
                        if e.encoding().is_some() {
                            let enc = s(us(e.encoding().unwrap()?.as_ref()));
                            println!("{}", enc);
                        }
                    }

                    // category

                    ,Event::Start(ref e) if !in_category && e.name() == b"category" => {
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
    }

    writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;

    // }}}
    
    // {{{ offers

    println!("---");
    println!("Start of offers");
    println!("---");

    let tag  = b"offers";
    writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())))?;

    for mid in &mids {
        
        println!("Start of {}", mid);

        let yml_ids: Vec<&str> = match export_file.is_through {
            true => export_file.categories.iter().flat_map(|c| &c.legacy_categories ).filter(|lc| lc.merchant_id.to_string().as_str()  == mid ).map(|lc| lc.yml_id.as_str() ).collect()
            ,false => export_file.legacy_categories.iter().filter(|lc| lc.merchant_id.to_string().as_str() == mid ).map( |lc| lc.yml_id.as_str() ).collect()
        };


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

            let mut img_no = 1;

            loop {
                let event = reader.read_event(&mut buf)?;

                match event {
                    Event::Decl(ref e) =>  {
                        if e.encoding().is_some() {
                            let enc = s(us(e.encoding().unwrap()?.as_ref()));
                            println!("{}", enc);
                        }
                    }

                    // offer

                    ,Event::Start(ref e) if !in_offer && e.name() == b"offer" => {
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
                        writer.write(offer_writer.into_inner().into_inner().as_ref())?;

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

                        let price_text: f32 = match price_text.trim().replace(',', ".").parse() {
                            Ok(f) => f
                            ,Err(_) => panic!("{}", price_text)
                        };

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

                        if export_file.partner_track_code.is_some() {
                            sid = format!("&sub_id={}", export_file.partner_track_code.as_ref().unwrap());
                        } else {
                            sid = s("");
                        }

                        let deep_link = format!("{}/cf/{}?mid={}&goto={}{}", 
                                                export_file.parked_domain.as_ref().unwrap_or(&ParkedDomain { name: s("https://f.gdeslon.ru") }).name, 
                                                api_token, 
                                                mid, 
                                                utf8_percent_encode(&url_text, FRAGMENT).to_string(),
                                                sid
                                               );

                        offer_writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())))?;
                        offer_writer.write_event(Event::CData(BytesText::from_plain_str(&deep_link)))?;
                        offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;

                        let tag = b"destination-url-do-not-send-traffic";

                        offer_writer.write_event(Event::Start(BytesStart::owned(tag.to_vec(), tag.len())))?;
                        offer_writer.write_event(Event::CData(BytesText::from_plain_str(&url_text)))?;
                        offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;

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

                        let mut pic = BytesStart::owned(tag.to_vec(), tag.len());
                        pic.push_attribute(("id", img_no.to_string().as_str()));

                        offer_writer.write_event(Event::Start(pic))?;
                        offer_writer.write_event(Event::CData(BytesText::from_plain_str(&format!("https://imgng.gdeslon.ru/mid/{}/imno/{}/cid/{}/hash/{}/{}.jpg", mid, im_no, article_text, small_hash, "big"))))?;
                        offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;

                        let tag = b"thumbnail";
                        let small_hash = hasher.result_str();

                        let mut pic = BytesStart::owned(tag.to_vec(), tag.len());
                        pic.push_attribute(("id", img_no.to_string().as_str()));

                        offer_writer.write_event(Event::Start(pic))?;
                        offer_writer.write_event(Event::CData(BytesText::from_plain_str(&format!("https://imgng.gdeslon.ru/mid/{}/imno/{}/cid/{}/hash/{}/{}.jpg", mid, im_no, article_text, small_hash, "small"))))?;
                        offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;
                        
                        let tag = b"original_picture";

                        let mut pic = BytesStart::owned(tag.to_vec(), tag.len());
                        pic.push_attribute(("id", img_no.to_string().as_str()));

                        offer_writer.write_event(Event::Start(pic))?;
                        offer_writer.write_event(Event::CData(BytesText::from_plain_str(&picture_text)))?;
                        offer_writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;

                        img_no += 1;
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


    }
   
    writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;

    // }}}

    let tag = b"shop";
    writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;
    
    let tag = b"yml_catalog";
    writer.write_event(Event::End(BytesEnd::borrowed(tag)))?;

    //println!("{}", s(us(writer.into_inner().into_inner().as_ref())));
    
    Ok(())
}

use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
