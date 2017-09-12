#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate duct;

extern crate clap;
use clap::{Arg, App};

extern crate tempdir;
extern crate select;

use tempdir::TempDir;
use tera::{Context, Tera};
use select::document::Document;
use select::predicate::Name;

use std::fs::File;
use std::io::Read;
use std::str;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Tera(tera::Error);
    }
}

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = compile_templates!("templates/**/*");
        tera.autoescape_on(vec![]);
        tera
    };
}

// Generate a base64 thumbnail from the given image
fn generate_thumbnail(image: &select::node::Node) -> Result<String> {
    let image_path = image.attr("data").ok_or(
        "data attribute not found for image",
    )?;

    let dir = TempDir::new("tmp")?;
    let thumb = dir.path().join("thumb.png");

    cmd!(
        "svgexport",
        image_path,
        &thumb,
        "\"svg{background:white;}\"",
        "30:",
        "0%"
    ).read()?;

    cmd!(
        "oxipng",
        &thumb,
        "--out",
        &thumb,
        "-o",
        "4",
        "--zopfli",
        "--strip",
        "all"
    ).read()?;

    let base64 = cmd!(
        "data-encoding",
        "--mode=encode",
        "--base",
        "64",
        "--input",
        &thumb
    ).read()?;

    let mut context = Context::new();
    context.add("object_html", &image.html());
    context.add("thumbnail_base64", &base64);
    Ok(TERA.render("image.html", &context)?)
}

fn run() -> Result<()> {


    let matches = App::new("lqip")
        .about("Does awesome things")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file")
                .takes_value(true),
        )
        .get_matches();

    let path = matches.value_of("input").ok_or("No input file given")?;
    let mut input = File::open(path)?;

    let mut dom = String::new();
    input.read_to_string(&mut dom)?;
    //println!("{:?}", dom);

    let document: Document = (*dom).into();

    let images = document.find(Name("object"));

    for image in images {
        let thumbnail = generate_thumbnail(&image)?;
        println!("{}", thumbnail);
        dom = dom.replace(&image.html(), &thumbnail);
    }

    //println!("{}", dom);



    Ok(())

    /*
    let buffered = BufReader::new(&f);

    for line in buffered.lines() {
        println!("{}", line?);
    }

    write!(f, "Rust\n💖\nFun")?;


    Ok(())
    */
}

quick_main!(run);