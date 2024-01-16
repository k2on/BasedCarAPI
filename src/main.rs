use std::{path::Path, fs::{File, create_dir_all}, io::{BufReader, BufWriter}, process::{exit, Command}};

use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};
use tokio::fs::{File as TFile, remove_file};
use tokio::io::AsyncWriteExt;
use futures::stream::StreamExt;


const BASE_URL: &str = "";


#[derive(Debug, Serialize, Deserialize)]
struct Car {
    colors: Vec<String>,
}

#[derive(Debug, Clone)]
enum Color {
    BLACK,
    WHITE,
    SILVER,
    BLUE,
    YELLOW,
    GREEN,
    RED,
    BAIGE,
    ORANGE
}

#[tokio::main]
async fn main() {
    let years = get_years(false).await;

    for year in years {
        println!("Year: {year}");
        let makes = get_makes(&year, false).await;

        for make in makes {
            println!("Make: {make}");
            let models = get_models(&year, &make, false).await;
            for model in models {
                println!("Model: {model}");

                let colors = get_colors(&year, &make, &model, false).await;
                println!("Colors: {colors:?}");
            }
        }
    }
}

impl Color {
    fn code(&self) -> i32 {
        match self {
            Color::BLACK => 1,
            Color::WHITE => 2,
            Color::SILVER => 4,
            Color::BLUE => 8,
            Color::YELLOW => 16,
            Color::GREEN => 32,
            Color::RED => 64,
            Color::BAIGE => 128,
            Color::ORANGE => 256,
        }
    }

    fn str(&self) -> String {
        match self {
            Color::BLACK => String::from("Black"),
            Color::WHITE => String::from("White"),
            Color::SILVER => String::from("Silver"),
            Color::BLUE => String::from("Blue"),
            Color::YELLOW => String::from("Yellow"),
            Color::GREEN => String::from("Green"),
            Color::RED => String::from("Red"),
            Color::BAIGE => String::from("Baige"),
            Color::ORANGE => String::from("Orange"),
        }
    }
}

async fn get_color(year: &str, make: &str, model: &str, color: &Color) -> Option<String> {
    let color_code = color.code();
    let url = format!("{year} {make} {model}");
    let resp = reqwest::get(url).await.unwrap();
    let html = resp.text().await.unwrap();
    let frag = Html::parse_fragment(&html);
    let selector = Selector::parse("#ImageGrid").unwrap();
    let img = frag.select(&selector).into_iter().nth(0);
    if let Some(el) = img {
        let sel = Selector::parse("img").unwrap();
        let mut i = el.select(&sel);

        let src = i.next().unwrap().value().attr("src");
        if let Some(src) = src {
            Some(format!("{src}"))
        } else {
            None
        }
    } else {
        None
    }
}

async fn get_years(force: bool) -> Vec<String> {
    get_data("data/years.json", "", "CarYears,", force).await
}

async fn get_makes(year: &str, force: bool) -> Vec<String> {
    get_data(&format!("data/years/{year}/makes.json"), &format!("?caryear={year}"), "CarMakes,", force).await
}

async fn get_models(year: &str, make: &str, force: bool) -> Vec<String> {
    get_data(&format!("data/years/{year}/{make}/models.json"), &format!("?caryear={year}&carmake={make}"), "CarModels,", force).await
}


async fn get_colors(year: &str, make: &str, model: &str, force: bool) -> Vec<String> {
    let colors_all: Vec<Color> = vec![
        Color::BLACK,
        Color::WHITE,
        Color::SILVER,
        Color::BLUE,
        Color::YELLOW,
        Color::GREEN,
        Color::RED,
        // Color::BAIGE,
        // Color::ORANGE,
    ];

    let path_str = format!("data/years/{year}/{make}/{model}/colors.json");
    let path = Path::new(&path_str);
    if path.exists() && !force {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    } else {
        let mut colors = Vec::new();

        for color in colors_all {
            println!("Getting {color:?}");
            if let Some(url) = get_color(&year, &make, &model, &color).await {
                let color_str = color.str();
                colors.push(color_str.clone());

                let path_str = format!("data/years/{year}/{make}/{model}/colors/{color_str}_base.jpeg");
                let path_out = format!("data/years/{year}/{make}/{model}/colors/{color_str}.png");
                let path = Path::new(&path_str);

                if let Some(parent) = path.parent() {
                    create_dir_all(parent).unwrap();
                }

                let mut file = TFile::create(path).await.unwrap();


                let resp = reqwest::get(url).await.unwrap();
                let mut stream = resp.bytes_stream();

                while let Some(item) = stream.next().await {
                    let chunk = item.expect("Failed to read chunk");
                    file.write_all(&chunk).await.expect("Failed to write to file");
                }

                println!("Removing bg...");

                let mut command = Command::new("rembg");
                command.arg("i").arg(path_str.clone()).arg(path_out);

                let output = command.output().expect("Failed to execute command");
                if output.status.success() {
                    println!("Background removed! Deleting original file...");

                    remove_file(path_str).await.unwrap();

                    println!("File deleted!");
                } else {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Error: {}", error_message);
                }

            }
        }
        
        if let Some(parent) = path.parent() {
            create_dir_all(parent).unwrap();
        }

        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &colors).unwrap();

        colors
    }
}


async fn get_data(path: &str, args: &str, first: &str, force: bool) -> Vec<String> {
    let path = Path::new(path);
    if path.exists() && !force {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    } else {
        let resp = reqwest::get(BASE_URL.to_owned() + args).await.unwrap();
        let html = resp.text().await.unwrap();
        let res_str = html.split_once(first).unwrap().1;
        let res = res_str.split(",").map(|s| s.to_owned()).collect();

        if let Some(parent) = path.parent() {
            create_dir_all(parent).unwrap();
        }

        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &res).unwrap();

        res
    }
}

// async fn get_models(year: &str, make: &str) -> Vec<String> {
//     let url = BASE_URL.to_owned() + "?caryear=" + year + "&carmake=" + make;
//     let resp = reqwest::get(url).await.unwrap();
//     let html = resp.text().await.unwrap();
//     let models_str = html.split_once("CarModels,").unwrap().1;
//     models_str.split(",").map(|s| s.to_owned()).collect()
// }

