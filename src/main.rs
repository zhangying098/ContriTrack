use chrono::{self, prelude};
use clap::{self, Parser};
use reqwest;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use std::{io::BufRead, path::PathBuf};
use xlsxwriter::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct PullRequest {
    org: String,
    repo: String,
    #[serde(rename = "ref")]
    reference: String,
    sig: String,
    link: String,
    state: String,
    author: String,
    assignees: String,
    created_at: String,
    updated_at: String,
    title: String,
    labels: String,
    draft: bool,
    mergeable: bool,
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct QueryPullParam {
    /// 提交者
    #[arg(short, long)]
    author: Option<String>,
    /// Pull Request的状态
    #[arg(short, long)]
    state: Option<String>,
    /// 近 x 天提交数据
    #[arg(short, long)]
    duration: u64,
    /// 是否保存为excel文件
    #[arg(short, long)]
    exc: bool,
    /// 批量收集提交者pr
    #[arg(short, long, value_name = "FILE")]
    inventory: Option<PathBuf>,
}

impl QueryPullParam {
    // url 的拼接
    fn url_splice(link: &str, arg: &QueryPullParam, user: Option<String>) -> String {
        let mut url = String::from(link);

        match user {
            Some(u) => url = url + "author=" + &u,
            None => {
                match arg.author.as_deref() {
                    Some(author) => url = url + "author=" + author,
                    None => {
                        panic!("None author infonation")
                    }
                };
            }
        }

        match arg.state.as_deref() {
            Some(state) => url = url.to_owned() + "&state=" + state,
            None => {
                panic!("None state infonation")
            }
        };

        url
    }

    // 保存为 excel
    fn do_excel(pull_list: Vec<PullRequest>) -> Result<(), XlsxError> {
        if pull_list.len() == 0 {
            eprintln!("无数据");
            return Ok(());
        }
        let filename = (&pull_list[0].author).to_string() + ".xlsx";

        let workbook = Workbook::new(&filename)?;
        let mut sheet = workbook.add_worksheet(None)?;

        let titles = ["author", "sig", "repo", "link", "created_at"];
        for (index, value) in titles.iter().enumerate() {
            sheet.write_string(
                0,
                index.try_into().unwrap(),
                *value,
                Some(&Format::new().set_font_color(FormatColor::Red)),
            )?;
        }

        for (index, value) in pull_list.iter().enumerate() {
            let row = index as u32 + 1;
            sheet.write_string(row, 0, &value.author, None)?;
            sheet.write_string(row, 1, &value.sig, None)?;
            sheet.write_string(row, 2, &value.repo, None)?;
            sheet.write_url(
                row,
                3,
                &value.link,
                Some(
                    &Format::new()
                        .set_font_color(FormatColor::Blue)
                        .set_underline(FormatUnderline::Single),
                ),
            )?;
            sheet.write_string(row, 4, &value.created_at, None)?;
        }
        workbook.close()?;

        Ok(())
    }

    // 批量信息获取
    fn get_inventory(path: std::path::PathBuf) -> Vec<String> {
        let mut inventory: Vec<String> = Vec::new();
        // 读取文件内容
        let file = std::fs::File::open(path).expect("Failed to open the file");
        let reader = std::io::BufReader::new(file);

        for item in reader.lines() {
            if let Ok(line) = item {
                inventory.push(line);
            }
        }
        inventory
    }

    // 参数合规检查
    fn param_check(arg: &QueryPullParam) {
        let inv = match &arg.inventory {
            Some(_t) => true,
            None => false,
        };

        let aut = match &arg.author {
            Some(_t) => true,
            None => false,
        };

        if aut && inv || !aut && !inv {
            panic!("--author --inventory conflict");
        }
    }

    async fn query_pr_info() -> Result<(), Box<dyn std::error::Error>> {
        let arg = QueryPullParam::parse();
        Self::param_check(&arg);

        let mut excel_vec: Vec<PullRequest> = Vec::new();
        let user_inventory: Vec<String> = match &arg.inventory {
            Some(t) => Self::get_inventory(t.to_path_buf()),
            None => {
                let aut = match &arg.author {
                    Some(t) => t,
                    None => panic!("--author --inventory not exist"),
                };
                vec![aut.to_string()]
            }
        };

        // 计算开始时间
        let duration = chrono::naive::Days::new(arg.duration);
        let now = prelude::Local::now();
        let ago = now.checked_sub_days(duration);
        let next = match ago {
            Some(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => panic!("fatal: duration overflow"),
        };

        // 信息采集主要实现
        for user in user_inventory {
            let url = Self::url_splice("https://ipb.osinfra.cn/pulls?", &arg, Some(user));
            if arg.exc {
                println!("请等待....");
            }
            for i in 1..100 {
                let url_real = format!("{}&page={}", url, i.to_string().as_str());
                let body: String = reqwest::get(url_real).await?.text().await?;
                let json: serde_json::Value = serde_json::from_str(&body).unwrap();
                if json["data"] != Value::Null {
                    let pr_data: Vec<PullRequest> =
                        serde_json::from_value(json["data"].clone()).unwrap();
                    for item in pr_data {
                        if item.created_at < next {
                            break;
                        }
                        if arg.exc {
                            excel_vec.push(item);
                        } else {
                            println!(
                                "{}, {}, {}, {}, {}",
                                item.author, item.sig, item.repo, item.link, item.created_at,
                            );
                        }
                    }
                } else {
                    break;
                }
            }
        }

        if arg.exc {
            Ok(Self::do_excel(excel_vec)?)
        } else {
            Ok(())
        }
    }
}

fn main() {
    let _ = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(QueryPullParam::query_pr_info());
}
