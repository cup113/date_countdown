use chrono::{Local, NaiveDate};
use once_cell::sync::Lazy;
use std::{fs::File, io::BufReader, path::PathBuf};

static PROGRAM_DIR: Lazy<PathBuf> = Lazy::new(|| {
    use std::env;
    let current_exe = env::current_exe().expect("Failed to get program directory");
    current_exe.parent().unwrap().to_path_buf()
});

fn main() {
    use std::process;
    DateCountDown::new().run().unwrap_or_else(|err_msg| {
        eprintln!("{}", err_msg);
        process::exit(1);
    });
}

struct DateCountDown {
    today: NaiveDate,
    date_file_path: PathBuf,
}

impl DateCountDown {
    fn new() -> DateCountDown {
        DateCountDown {
            today: Local::now().date_naive(),
            date_file_path: PROGRAM_DIR.join("dates.txt"),
        }
    }

    fn get_file(&self) -> Result<BufReader<File>, String> {
        let date_file = File::options()
            .read(true)
            .open(&self.date_file_path)
            .or_else(|err| {
                Err(format!(
                    "Failed to open file \"{}\": {}",
                    self.date_file_path.display(),
                    err
                ))
            })?;
        Ok(BufReader::new(date_file))
    }

    fn parse_line(&self, line: &str, lineno: usize) -> Result<String, String> {
        use std::cmp::Ordering;
        use std::str::FromStr;
        // Line is the pattern of: {yyyy}/{m}/{d} {some-content}
        let (date_part, content) = match line.split_once(' ') {
            Some(parts) => parts,
            None => return Err(format!("Failed to parse line {}: No spaces found.", lineno)),
        };
        let date_part = date_part.replace('/', "-");
        let date = NaiveDate::from_str(&date_part).map_err(|err| {
            format!(
                "Failed to parse line {}: Illegal date expression {}: {}.",
                lineno, date_part, err
            )
        })?;
        let days_diff = (date - self.today).num_days();
        let line_printed: String = match days_diff.cmp(&0) {
            Ordering::Equal => format!("{} 就在今天", content),
            Ordering::Greater => format!("距离 {} 还有 {} 天", content, days_diff),
            Ordering::Less => format!("距离 {} 已经过去了 {} 天", content, -days_diff),
        };
        Ok(line_printed)
    }

    fn run(self) -> Result<(), String> {
        use std::io::BufRead;
        let file = self.get_file()?;
        let lines = file.lines();
        for (i, line) in lines.enumerate() {
            let line = line.map_err(|err| {
                format!(
                    "Failed to read file \"{}\":{}: {}",
                    self.date_file_path.display(),
                    i + 1,
                    err
                )
            })?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            println!("{}", self.parse_line(&line, i + 1)?);
        }
        Ok(())
    }
}
