use hyper::{header::CONTENT_TYPE, rt::Future, service::service_fn_ok, Body, Response, Server};
use prometheus::{Encoder, IntGaugeVec, TextEncoder};
use std::net::SocketAddr;

lazy_static! {
    static ref TOTAL_RECV_MESSAGES_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "messages_recv_total",
        "total messages received from source by target from last request",
        &["source", "target", "status"]
    )
    .unwrap();
}

pub fn start_server(address: String) {
    let addr: SocketAddr = address.parse().unwrap();
    let new_service = || {
        let encoder = TextEncoder::new();
        service_fn_ok(move |_request| {
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let response = Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap();
            TOTAL_RECV_MESSAGES_GAUGE.reset();
            response
        })
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| log::error!("Server error: {}", e));

    log::info!("Metrics are exposed under: {:?}", addr);
    hyper::rt::run(server);
}

pub fn inc_received_messages(from: String, target: String, status: String) {
    TOTAL_RECV_MESSAGES_GAUGE
        .with_label_values(&[from.as_str(), target.as_str(), status.as_str()])
        .inc();
}
