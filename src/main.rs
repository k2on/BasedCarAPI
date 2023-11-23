use std::{path::Path, fs::{File, create_dir_all}, io::{BufReader, BufWriter}};

use serde::{Serialize, Deserialize};

const BASE_URL: &str = "https://www.evoxstock.com/ajaxpickyourvehicle.asp";


#[derive(Debug, Serialize, Deserialize)]
struct Years {
    years: Vec<String>,
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
            println!("Models: {models:?}");
            // for model in models {
            //     println!("Model: {model}");
            // }

        }
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

