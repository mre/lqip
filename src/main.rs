#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate lazy_static;

extern crate select;

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
    // svgexport clothes.svg clothes-min20.png "svg{background:white;}" 30: 90% && wc -c < clothes-min20.png
    // oxipng clothes-min20.png --out clothes-oxi.png -o 4 --zopfli --strip all
    // data-encoding --mode=encode --base 64 --input clothes-oxi.png



    let mut context = Context::new();
    context.add("object_html", &image.html());
    context.add("thumbnail_base64", &"0xDEADBEEF".to_string());
    Ok(TERA.render("image.html", &context)?)
}

fn run() -> Result<()> {
    let path = "2017-makefiles.1.md";
    let mut input = File::open(path)?;

    let mut dom = String::new();
    input.read_to_string(&mut dom)?;
    println!("{:?}", dom);

    let document: Document = (*dom).into();

    let images = document.find(Name("object"));

    for image in images {
        let thumbnail = generate_thumbnail(&image)?;
        println!("{}", thumbnail);
        dom = dom.replace(&image.html(), &thumbnail);
    }

    println!("{}", dom);



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