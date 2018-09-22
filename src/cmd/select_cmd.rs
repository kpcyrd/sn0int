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
        Args::Domains(filter) => show_domains(rl, &filter),
        Args::Subdomains(filter) => show_subdomains(rl, &filter),
        Args::IpAddrs(filter) => show_ipaddrs(rl, &filter),
        Args::Urls(filter) => show_urls(rl, &filter),
    }
}

fn show_domains(rl: &mut Readline, filter: &Filter) -> Result<()> {
    for domain in rl.db().filter::<Domain>(&filter.parse()?)? {
        println!("{:#?}", domain);
    }

    Ok(())
}

fn show_subdomains(rl: &mut Readline, filter: &Filter) -> Result<()> {
    for subdomain in rl.db().filter::<Subdomain>(&filter.parse()?)? {
        println!("{:#?}", subdomain);
    }

    Ok(())
}

fn show_ipaddrs(rl: &mut Readline, filter: &Filter) -> Result<()> {
    for ipaddr in rl.db().filter::<IpAddr>(&filter.parse()?)? {
        println!("{:#?}", ipaddr);
    }

    Ok(())
}

fn show_urls(rl: &mut Readline, filter: &Filter) -> Result<()> {
    for url in rl.db().filter::<Url>(&filter.parse()?)? {
        println!("{:?}", url);
    }

    Ok(())
}
