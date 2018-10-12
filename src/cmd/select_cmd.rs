use errors::*;

use db;
use shell::Readline;
use structopt::StructOpt;
use models::*;


#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="domains")]
    Domains(Filter),
    #[structopt(name="subdomains")]
    Subdomains(Filter),
    #[structopt(name="ipaddrs")]
    IpAddrs(Filter),
    #[structopt(name="urls")]
    Urls(Filter),
    #[structopt(name="emails")]
    Emails(Filter),
}

#[derive(Debug, StructOpt)]
pub struct Filter {
    args: Vec<String>,
}

impl Filter {
    pub fn parse(&self) -> Result<db::Filter> {
        db::Filter::parse(&self.args)
    }
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;
    match args {
        Args::Domains(filter) => select::<Domain>(rl, &filter),
        Args::Subdomains(filter) => select::<Subdomain>(rl, &filter),
        Args::IpAddrs(filter) => select::<IpAddr>(rl, &filter),
        Args::Urls(filter) => select::<Url>(rl, &filter),
        Args::Emails(filter) => select::<Email>(rl, &filter),
    }
}

fn select<T: Model + Detailed>(rl: &mut Readline, filter: &Filter) -> Result<()> {
    for obj in rl.db().filter::<T>(&filter.parse()?)? {
        println!("{}", obj.detailed(rl.db())?);
    }

    Ok(())
}
