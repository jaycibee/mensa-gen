use reqwest::blocking::get;
use scraper::{Html, Selector, ElementRef};
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use tera::{Tera, Context};
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug, Copy, Clone, Serialize)]
enum Kind {
    Normal,
    Vegetarian,
    Vegan
}

#[derive(Debug, Clone, Serialize)]
struct Location {
    pub name: String,
    pub meals: Vec<Meal>
}

#[derive(Debug, Clone, Serialize)]
struct Meal {
    pub name: String,
    pub price: (String, String, String),
    pub kind: Kind
}

fn parse_speiseplan(document: Html) -> Vec<Location> {
    lazy_static! {
        static ref SELECTOR: Selector = Selector::parse("div.tx-epwerkmenu-menu-location-container:not(.d-none)").unwrap();
    }
    document.select(&SELECTOR).map(|l| parse_location(l)).collect()
}

fn parse_location(location: ElementRef) -> Location {
    lazy_static! {
        static ref SELECTOR_NAME: Selector = Selector::parse(".mensainfo__title").unwrap();
        static ref SELECTOR_MEAL: Selector = Selector::parse(".singlemeal").unwrap();
    }
    let name = location.select(&SELECTOR_NAME).next().unwrap().inner_html().trim().to_string();
    let meals = location.select(&SELECTOR_MEAL).map(|m| parse_meal(m)).collect();
    Location { name, meals }
}

fn parse_meal(meal: ElementRef) -> Meal {
    lazy_static! {
        static ref SELECTOR_NAME: Selector = Selector::parse(".singlemeal__headline").unwrap();
        static ref SELECTOR_ICONS: Selector = Selector::parse(".singlemeal__icon").unwrap();
        static ref SELECTOR_PRICE: Selector = Selector::parse(".singlemeal__info--semibold").unwrap();
    }
    let name = meal.select(&SELECTOR_NAME).next().unwrap().inner_html().trim().to_string();
    let prices_raw = meal.select(&SELECTOR_PRICE).map(|t| t.inner_html().trim().to_string());
    let mut prices = prices_raw.clone().filter(|s| s.contains("€")).map(|s| s.split_whitespace().next().unwrap().to_string());
    let price = (prices.next().unwrap(), prices.next().unwrap(), prices.next().unwrap());
    let mut icons = meal.select(&SELECTOR_ICONS).filter_map(|t| t.attr("src"));
    let kind = if icons.clone().any(|e| e == "/typo3conf/ext/ep_werk_menu/Resources/Public/Icons/MealIcons/Normal/vega.svg") {
        Kind::Vegan
    } else if icons.any(|e| e == "/typo3conf/ext/ep_werk_menu/Resources/Public/Icons/MealIcons/Normal/vege.svg") {
        Kind::Vegetarian
    } else {
        Kind::Normal
    };
    Meal { name, price, kind }
}

#[derive(Debug, Clone, Parser)]
#[command(author="Helena Jäger <hi@elli.gay>")]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    locations: Option<PathBuf>,
    template: PathBuf,
    output: PathBuf
}

fn main() {
    let args = Args::parse();
    let mut tera = Tera::default();
    if let Err(_) = tera.add_template_file(args.template.clone(), Some("plan")) {
        eprintln!("error: could not open template {:?}", args.template);
        return;
    }
    let mut file = match File::create(args.output.clone()) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("error: could not open file {:?}", args.output);
            return;
        }
    };
    let location_filter: Box<dyn Fn(&Location) -> bool> = match args.locations.clone() {
        None => Box::new(|_| true),
        Some(l) => match File::open(l) {
            Ok(f) => {
                let mensen = io::BufReader::new(f).lines().filter_map(|l| l.ok()).map(|l| l.trim().to_string()).collect::<Vec<String>>();
                Box::new(move |l| {mensen.contains(&l.name)})
            },
            Err(_) => {
                eprintln!("error: could not open file {:?}", args.locations);
                return;
            }
        }
    };
    let raw = get("https://www.stwhh.de/speiseplan?t=today").unwrap().text().unwrap();
    let document = Html::parse_document(&raw);
    let speiseplan = parse_speiseplan(document).into_iter().filter(location_filter).collect::<Vec<Location>>();
    let mut context = Context::new();
    context.insert("locations", &speiseplan);
    tera.render_to("plan", &context, file);
}
