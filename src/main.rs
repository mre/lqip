#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate duct;

extern crate clap;
extern crate tera;

use clap::{Arg, App};

extern crate tempdir;
extern crate select;

use tempdir::TempDir;
use tera::{Context, Tera};
use select::document::Document;
use select::predicate::Name;

use std::fs::File;
use std::io::{Read, Write};
use std::str;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Tera(tera::Error);
    }
}

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("image.html", r#"<div class="loader">
            {{ object_html }}
            <img class="frozen" src="data:image/png;base64,{{ thumbnail_base64 }}" />
        </div>"#).unwrap();
        tera.autoescape_on(vec![]);
        tera
    };
}

// Generate a base64 thumbnail from the given image
fn generate_thumbnail(
    image: &select::node::Node,
    dimensions: &str,
    quality: &str,
) -> Result<String> {
    let mut image_path = image.attr("data").ok_or(
        "data attribute not found for image",
    )?;

    // Awkward way to create a relative path
    if image_path.starts_with('/') {
        image_path = &image_path[1..];
    }

    let dir = TempDir::new("tmp")?;
    let thumb = dir.path().join("thumb.png");

    cmd!(
        "svgexport",
        image_path,
        &thumb,
        r#"svg{background:white;}"#,
        dimensions,
        quality
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
                .help("Input filename")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dimensions")
                .short("d")
                .long("dimensions")
                .help("Thumbnail dimensions (e.g. 30:")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("quality")
                .short("q")
                .long("quality")
                .help("Thumbnail quality (e.g. 10%")
                .takes_value(true),
        )
        .get_matches();

    let path = matches.value_of("input").ok_or("No input file given")?;
    let dimensions = matches.value_of("dimensions").unwrap_or("30:");
    let quality = matches.value_of("quality").unwrap_or("1%");

    let mut file = File::open(path)?;
    let mut dom = String::new();
    file.read_to_string(&mut dom)?;

    let document: Document = (*dom).into();

    let images = document.find(Name("object"));

    for image in images {
        let thumbnail = generate_thumbnail(&image, dimensions, quality)?;
        dom = dom.replace(&image.html(), &thumbnail);
    }

    let mut output = File::create(path)?;
    output.write_all(dom.as_bytes())?;

    Ok(())
}

quick_main!(run);