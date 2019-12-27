use std::io::{Cursor, BufReader};
use std::fs::File;

use quick_xml::{Reader, Writer};
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};

extern crate glob;
use glob::glob;

type WCU8 = Writer<Cursor<Vec<u8>>>;

fn s(s: &str) -> String {
    s.to_string()
}

fn us(s: &[u8]) -> &str {
    std::str::from_utf8(&s).unwrap_or("")
}

fn set_value(value: &str, writer: &mut WCU8) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::CData(BytesText::from_plain_str(value)))?;
    Ok(())
}

fn start_tag(name: &[u8], writer: &mut WCU8) {
    writer.write_event(Event::Start(BytesStart::owned(name.to_vec(), name.len())));
}

fn end_tag(name: &[u8], writer: &mut WCU8) {
    writer.write_event(Event::End(BytesEnd::borrowed(name)));
}

fn create_tag(name: &[u8], text: &str, writer: &mut WCU8) -> Result<(), Box<dyn std::error::Error>> {
    start_tag(name, writer);
    set_value(text, writer)?;
    end_tag(name, writer);

    Ok(())
}

fn start_handler(e: &BytesStart, w: &mut WCU8) { 
    match e.name() {
        b"picture" => start_tag(b"bibi", w)
        ,b"url"  => start_tag(e.name(), w)
        ,_ => { w.write_event(Event::Start(e.clone())).unwrap(); }
    }
}

type RBF  = Reader<BufReader<File>>;

fn text_handler(name: &Vec<u8>, e: &BytesText, w: &mut WCU8, ef: &ExportFile, r: &mut Reader) {
    match name.as_slice() {
        b"price" => {  println!("{}", ef.min_price.unwrap_or(1.0));  }
        ,b"picture" => { w.write_event(Event::CData(BytesText::from_plain_str("test"))).unwrap(); }

        ,b"url" => {
            let value = e.unescape_and_decode(r).unwrap();
            create_tag(b"destination-url-do-not-send-traffic", value, w);
        }

        ,_ => { w.write_event(Event::Text(e.clone())).unwrap(); }
    }
}

fn end_handler(e: &BytesEnd, w: &mut WCU8) { 
    match e.name() {
        b"picture" => end_tag(b"bibi", w)
        ,b"url"  => end_tag(e.name(), w)
        ,_ => { w.write_event(Event::End(e.clone())).unwrap(); }
    }
}

fn read_one(path: &std::path::Path, tag: &[u8], yml_ids: Vec<&str>, writer: &mut WCU8, ef: &ExportFile)  -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = Reader::from_file(path)?;
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut in_tag = false;
    let mut name: Vec<u8> = Vec::new();

    loop {
        let event = reader.read_event(&mut buf)?;

        match event {
            Event::Start(ref e) if in_tag && e.name() != tag => { 
                name.clear();
                name = e.name().to_vec();
                start_handler(e, writer)
            }

            ,Event::Text(ref e) | Event::CData(ref e) if in_tag  => text_handler(&name, e, writer, ef, &reader)
            ,Event::End(ref e) if in_tag && e.name() != tag => end_handler(e, writer)
            
            ,Event::Start(ref e) if !in_tag && e.name() == tag  => {
                in_tag = true;
                writer.write_event(event)?;
            }

            ,Event::Empty(ref e) if in_tag  => {
                writer.write_event(event)?;
            }
            
            ,Event::End(ref e) if in_tag && e.name() == tag  => {
                in_tag = false;
                writer.write_event(event)?;
                break
            }
            
            ,Event::Eof => break

            ,_ => ()
        }

        buf.clear();
    }

    Ok(())
}

fn glob_loop(wrap: &[u8], tag: &[u8], w: &mut WCU8, ef: &ExportFile) -> Result<(), Box<dyn std::error::Error>> {
    start_tag(wrap, w);

    let glob_paths = glob(&format!("/i4/slon-i4-downloader/xmls/{}/*.xml", "80549"))?;

    for path in glob_paths {
        let p = path?;
        read_one(p.as_path(), tag, vec!["756"], w, ef);
    }

    end_tag(wrap, w);

    Ok(())
}

struct ExportFile {
    min_price: Option<f32>
    ,max_price: Option<f32>
}

fn init_export_file() -> ExportFile {
    ExportFile {
        min_price: Some(10.0)
        ,max_price: Some(1000.0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ef = init_export_file();
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    start_tag(b"yml_catalog", &mut writer);
    start_tag(b"shop", &mut writer);

    glob_loop(b"categories", b"category", &mut writer, &ef)?;
    glob_loop(b"offers", b"offer", &mut writer, &ef)?;
   
    end_tag(b"shop", &mut writer);
    end_tag(b"yml_catalog", &mut writer);

    println!("{}", us(writer.into_inner().into_inner().as_ref()));

    Ok(())
}


