use clap::Parser;
use regex::Regex;
use serde::ser::{self, SerializeMap};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::Command,
};
use tempfile::tempdir;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = String::from("w30759.pdf"))]
    src: String,

    #[arg(long, default_value_t=String::from("data.json"))]
    json_output: String,

    #[arg(long, default_value_t=String::from("data.tsv"))]
    tsv_output: String,
}

fn normalize_country(name: &str) -> String {
    match name {
        "Bosnia" | "Bosnia Herzegovina" => "Bosnia and Herzegovina",
        "CAR" => "Central African Republic",
        "North. Cyprus" => "Northern Cyprus",
        "Nagorno Karabakh" => "Nagorno-Karabash",
        "Gambia" => "The Gambia",
        "Trinidad &Tobago" | "Trinidad & Tobago" => " Trinidad and Tobago",
        "UAEs" => "UAE",
        _ => name,
    }
    .to_owned()
}

fn table<R: Read>(r: R, pat: &Regex) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut entries = Vec::new();
    for line in BufReader::new(r).lines() {
        let line = line?;
        let start = match pat.find(&line) {
            Some(m) => m.start(),
            None => continue,
        };
        let data = line[start..]
            .split(" ")
            .map(|v| v.parse::<i32>())
            .collect::<Result<Vec<_>, _>>()?;
        entries.push(Entry {
            country: normalize_country(line[..start].trim()),
            values: data,
        });
    }
    Ok(entries)
}

fn process_pdf_text<P: AsRef<Path>, T, F>(
    src: P,
    first_page: usize,
    last_page: usize,
    op: F,
) -> Result<T, Box<dyn Error>>
where
    F: FnOnce(fs::File) -> Result<T, Box<dyn Error>>,
{
    let dir = tempdir()?;
    let dst = dir.as_ref().join("pdf.txt");
    Command::new("pdftotext")
        .args(&[
            "-raw",
            "-f",
            &format!("{}", first_page),
            "-l",
            &format!("{}", last_page),
            src.as_ref()
                .to_str()
                .ok_or_else(|| String::from("invalid src path"))?,
            dst.to_str()
                .ok_or_else(|| String::from("invalid dst path"))?,
        ])
        .status()?;
    op(fs::File::open(&dst)?)
}

#[derive(Debug)]
struct Entry {
    country: String,
    values: Vec<i32>,
}

impl ser::Serialize for Entry {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let cols = &[
            "cantril",
            "enjoy",
            "smile",
            "well_rested",
            "pain",
            "sadness",
            "worry",
            "anger",
            "positive",
            "negative",
            "final",
        ];
        let mut sm = s.serialize_map(Some(self.values.len() + 1))?;
        sm.serialize_entry("country", &self.country)?;
        let cols = match self.values.len() {
            8 => &cols[..8],
            3 => &cols[8..],
            _ => cols,
        };
        for (i, key) in cols.iter().enumerate() {
            sm.serialize_entry(key, &self.values[i])?;
        }
        sm.end()
    }
}

fn join(a: &[Entry], b: &[Entry]) -> Result<Vec<Entry>, Box<dyn Error>> {
    let idx: HashMap<&str, &Entry> = a
        .iter()
        .map(|v| (v.country.as_str(), v))
        .collect::<HashMap<&str, &Entry>>();
    let mut joined = Vec::with_capacity(a.len());
    for b in b.iter() {
        let a = idx
            .get(b.country.as_str())
            .ok_or_else(|| format!("missing key for {}", b.country))?;
        let mut values = Vec::with_capacity(a.values.len() + b.values.len());
        for v in a.values.iter() {
            values.push(*v);
        }
        for v in b.values.iter() {
            values.push(*v);
        }
        joined.push(Entry {
            country: b.country.to_owned(),
            values: values,
        });
    }
    Ok(joined)
}

fn tsv_to_writer<W: Write>(w: &mut W, data: &[Entry]) -> Result<(), Box<dyn Error>> {
    let cols = &[
        "CANTRIL",
        "ENJOY",
        "SMILE",
        "WELL-RESTED",
        "PAIN",
        "SADNESS",
        "WORRY",
        "ANGER",
        "POSITIVE",
        "NEGATIVE",
        "FINAL",
    ];

    let n = data
        .first()
        .ok_or_else(|| String::from("empty data"))?
        .values
        .len();
    let cols = match n {
        8 => &cols[..8],
        3 => &cols[8..],
        11 => cols,
        _ => return Err("first entry has invalid number of columns".into()),
    };

    writeln!(w, "COUNTRY\t{}", cols.join("\t"))?;
    for (i, entry) in data.iter().enumerate() {
        if entry.values.len() != n {
            return Err(format!("entry {} has invalid number of values", i).into());
        }
        writeln!(
            w,
            "{}\t{}",
            entry.country,
            entry
                .values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("\t")
        )?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let table5 = process_pdf_text(&args.src, 25, 30, |r| {
        table(r, &Regex::new(r"\d+ \d+ \d+ \d+ \d+ \d+ \d+ \d+$")?)
    })?;

    let table8 = process_pdf_text(&args.src, 35, 39, |r| {
        table(r, &Regex::new(r"\d+ \d+ \d+$")?)
    })?;

    let data = join(&table5, &table8)?;
    serde_json::to_writer_pretty(fs::File::create(&args.json_output)?, &data)?;
    tsv_to_writer(&mut fs::File::create(&args.tsv_output)?, &data)?;

    Ok(())
}
