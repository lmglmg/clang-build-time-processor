use serde::Deserialize;

type Duration = u64;

#[derive(Deserialize)]
pub struct Args {
    pub detail: Option<String>,
}

#[derive(Deserialize)]
pub struct Event<'a> {
    pub pid: u64,
    pub tid: u64,
    pub ts: u64,
    pub name: &'a str,
    pub dur: Option<Duration>,
    pub args: Option<Args>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
#[allow(non_snake_case)]
pub struct Profile<'a> {
    pub traceEvents: Vec<Event<'a>>,
    pub beginningOfTime: u128,
}