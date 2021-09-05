#[macro_use] extern crate prometheus;
#[macro_use] extern crate lazy_static;

use prometheus::{TextEncoder, Encoder};

use warp::Filter;
use warp::http::StatusCode;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

lazy_static! {
    static ref REQUEST_COUNTER: prometheus::IntCounter =
        prometheus::register_int_counter!("request_counter", "Number of requests").unwrap();

    static ref IP_ADDR_GUAGE: prometheus::IntGaugeVec =
        prometheus::register_int_gauge_vec!(
                "interface_hashed_address",
                "IP Address as number",
                &["interface"]
        ).unwrap();
}

fn ip_to_int(ip: get_if_addrs::Interface) -> i64 {

    match ip.ip() {
        std::net::IpAddr::V4(v4addr) => {
            let octlets = v4addr.octets();
            i64::from_ne_bytes([0, 0, 0, 0, octlets[0], octlets[1], octlets[2], octlets[3]])
        },
        std::net::IpAddr::V6(v6addr) => {
        let mut hasher = DefaultHasher::new();
            v6addr.hash(&mut hasher);
            i64::from_ne_bytes(hasher.finish().to_ne_bytes())
        },
    }

}

fn metrics() -> impl warp::Reply {

    REQUEST_COUNTER.inc();

    let addrs = get_if_addrs::get_if_addrs().expect("interfaces");

    for addr in addrs {
        IP_ADDR_GUAGE.with_label_values(&[&addr.name])
            .set(i64::from_ne_bytes(ip_to_int(addr).to_ne_bytes()))
    }

    let encoder = TextEncoder::new();
    let mut metrics = Vec::new();
    let metric_families = prometheus::gather();


    if let Err(err) = encoder.encode(&metric_families, &mut metrics) {
        return warp::reply::with_status(format!("error: {}", err).into(), StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        return warp::reply::with_status(metrics, StatusCode::OK)
    }
}

#[tokio::main]
async fn main() {
    let metrics = warp::path("metrics")
        .and(warp::get())
        .map(metrics); 


    warp::serve(metrics)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
